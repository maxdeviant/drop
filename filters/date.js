'use strict';

var ejs = require('ejs');

ejs.filters.short = function (date) {
    return (date.getMonth() + 1) + '/' + date.getDate() + '/' + date.getFullYear().toString().slice(2, 4) + ' ' + date.getHours() + ':' + date.getMinutes();
};

module.exports = ejs.filters;
