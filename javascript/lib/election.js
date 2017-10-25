(function() {
    var lock = require('./lock.js');
    module.exports.Election = class Election {
        constructor(name, masterFn, lostMasterFn) {
            this.name = name;
            this.masterFn = masterFn;
            this.lostMasterFn = lostMasterFn;
            this.lock = new lock.Lock(name, this.lockAcquired.bind(this), this.lockLost.bind(this));
        }

        lockAcquired() {
            setTimeout(this.masterFn, 0);
        }

        lockLost() {
            setTimeout(this.lostMasterFn, 0);
        }

        run() {
            this.lock.lock();
        }

        shutdown() {
            this.lock.unlock();
        }
    };
})();