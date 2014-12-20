'use strict';

var mongoose = require('mongoose');

var HitSchema = mongoose.Schema({
    requester: {
        type: String,
        required: true,
    },
    timestamp: {
        type: Date
    },
    file: {
        type: String,
        required: true
    }
});

var Hit = mongoose.model('Hit', HitSchema);

module.exports = Hit;
