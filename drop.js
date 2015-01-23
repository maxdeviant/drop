'use strict';

var express = require('express');
var mongoose = require('mongoose');
var path = require('path');
var fs = require('fs');
var bodyParser = require('body-parser');
var jwt = require('jwt-simple');
var session = require('express-session');
var bcrypt = require('bcrypt');
var formidable = require('formidable');

var config = require('./config');
var jwtauth = require('./lib/jwt-auth');

var dateFilters = require('./filters/date.js');

mongoose.connect('mongodb://localhost/drop');

var Hit = require('./models/hit');

var app = express();

app.set('env', 'production');

app.set('jwtTokenSecret', config.token_secret);

app.use(session({
    secret: config.session_secret,
    saveUninitialized: true,
    resave: true
}));

app.use(bodyParser.json());
app.use(bodyParser.urlencoded({
    extended: false
}));

app.use(express.static(path.join(__dirname, 'public'), {
    maxAge: 24 * 60 * 60 * 1000
}));

app.set('views', path.join(__dirname, 'views'));
app.set('view engine', 'ejs');

var router = express.Router();

router.route('/')
    .get(function (req, res) {
        return res.render('index');
    });

router.route('/login')
    .get(function (req, res) {
        return res.render('login');
    });

router.route('/auth')
    .post(function (req, res) {
        var username = req.body.username;
        var password = req.body.password;

        if (username === config.user.username) {
            bcrypt.compare(password, config.user.password, function (err, isMatch) {
                if (err) {
                    return res.redirect('/login');
                }

                var expires = new Date();
                expires.setDate(expires.getDate() + 7);

                var token = jwt.encode({
                    username: username,
                    expires: expires
                }, app.get('jwtTokenSecret'));

                req.session.token = token;

                return res.redirect('/upload');
            });
        }
    });

router.route('/upload')
    .get([jwtauth], function (req, res) {
        return res.render('upload');
    })
    .post([jwtauth], function (req, res) {
        var form = new formidable.IncomingForm();

        var redirectUrl = '';

        form.uploadDir = config.storage_root + '/' + config.subdir;
        form.keepExtensions = true;
        form.hash = 'md5';

        if (!fs.exists(form.uploadDir)) {
            fs.mkdir(form.uploadDir);
        }

        form.on('file', function (field, file) {
            fs.rename(file.path, path.join(form.uploadDir, file.hash + getExtension(file.path)));

            redirectUrl = path.join('/', config.subdir, file.hash + getExtension(file.path));
        });

        form.parse(req, function (err, fields, files) {
            return res.redirect(redirectUrl);
        });
    });

function getExtension(path) {
    return path.slice(path.lastIndexOf('.'), path.length);
}

router.route('/stats')
    .get(function (req, res) {
        Hit.find({}, function (err, hits) {
            var stats = {};

            hits.forEach(function (hit, index) {
                var file = hit.file;
                var timestamp = new Date(hit.timestamp);

                stats[file] = stats[file] || {
                    fileName: file,
                    lastHit: null
                };

                if (typeof stats[file].hits === 'undefined') {
                    stats[file].hits = 1;
                } else {
                    stats[file].hits++;
                }

                if (stats[file].lastHit < timestamp) {
                    stats[file].lastHit = timestamp;
                }
            });

            return res.render('stats', {
                stats: stats
            });
        });
    });

router.route('/stats/unique')
    .get(function (req, res) {
        Hit.find({}, function (err, hits) {
            var stats = {};
            var visited = {};

            hits.forEach(function (hit, index) {
                var file = hit.file;
                var timestamp = new Date(hit.timestamp);
                var requester = hit.requester;

                stats[file] = stats[file] || {
                    fileName: file,
                    lastHit: null
                };

                visited[file] = visited[file] || {};

                if (typeof visited[file][requester] === 'undefined') {
                    visited[file][requester] = false;
                }

                if (!visited[file][requester]) {
                    if (typeof stats[file].hits === 'undefined') {
                        stats[file].hits = 1;
                    } else {
                        stats[file].hits++;
                    }

                    visited[file][requester] = true;
                }

                if (stats[file].lastHit < timestamp) {
                    stats[file].lastHit = timestamp;
                }
            });

            return res.json(stats);
        });
    });

router.route('*')
    .get(function (req, res) {
        var filePath = req.params[0];

        if (/\/{2,}/.test(filePath)) {
            return res.redirect(filePath.replace(/\/{2,}/g, '/'));
        }

        var ip = req.headers['x-real-ip'] ||
            req.headers['x-forwarded-for'] ||
            req.connection.remoteAddress ||
            req.socket.remoteAddress ||
            req.connection.socket.remoteAddress;

        fs.exists(path.join(config.storage_root, filePath), function (exists) {
            if (!exists) {
                return res.render('file-not-found');
            }

            if (!req.headers['referer'] || req.headers['host'].indexOf('localhost') === -1) {
                var hit = new Hit();

                hit.requester = ip;
                hit.timestamp = Date.now();
                hit.file = filePath;

                hit.save(function (err) {
                    if (err) {
                        console.log(err);
                    }

                    return res.sendFile(path.join(config.storage_root, filePath));
                });
            } else {
                return res.sendFile(path.join(config.storage_root, filePath));
            }
        });
    });

app.use('/', router);

app.use(function (err, req, res, next) {
    res.status(err.status || 500);
    return res.render('error', {
        message: err.message,
        error: {
            status: err.status
        }
    });
});

app.set('port', process.env.PORT || 3000);

var server = app.listen(app.get('port'), function () {
    console.log('滴: Server listening on port ' + server.address().port);
});
