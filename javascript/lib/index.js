(function() {
    var lock = require('./lock.js');
    module.exports.Lock = lock.Lock;
    module.exports.debug = lock.debug;

    var election = require('./election.js');
    module.exports.Election = election.Election;
})();