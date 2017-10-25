(function() {
    var lock = require('./lock.js');
    module.exports.Lock = lock.Lock;

    var election = require('./election.js');
    module.exports.Election = election.Election;
})();