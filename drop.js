'use strict';

var express = require('express');
var mongoose = require('mongoose');
var path = require('path');
var fs = require('fs');
var bodyParser = require('body-parser');

var config = require('./config');

mongoose.connect('mongodb://localhost/drop');

var Hit = require('./models/hit');

var app = express();

app.set('env', 'production');

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

            return res.json(stats);
        });
    });

router.route('*')
    .get(function (req, res) {
        var filePath = req.params[0];

        var ip = req.headers['x-forwarded-for'] ||
            req.connection.remoteAddress ||
            req.socket.remoteAddress ||
            req.connection.socket.remoteAddress;

        fs.exists(path.join(config.storageRoot, filePath), function (exists) {
            if (exists) {
                var hit = new Hit();

                hit.requester = ip;
                hit.timestamp = Date.now();
                hit.file = filePath;

                hit.save(function (err) {
                    if (err) {
                        console.log(err);
                    }

                    res.sendFile(path.join(config.storageRoot, filePath));
                });
            } else {
                res.render('file-not-found');
            }
        });

    });

app.use('/', router);

app.use(function (err, req, res, next) {
    res.status(err.status || 500);
    res.render('error', {
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
