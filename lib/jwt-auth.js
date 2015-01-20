'use strict';

var jwt = require('jwt-simple');

var jwtauth = function (req, res, next) {
    var token = req.session.token;

    if (token) {
        try {
            var decoded = jwt.decode(token, app.get('jwtTokenSecret'));

            if (decoded.expires <= Date.now()) {
                return res.redirect('/login');
            }
        } catch (err) {
            return next();
        }
    } else {
        return res.redirect('/login');
    }
}

module.exports = jwtauth;
