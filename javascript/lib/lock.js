(function() {
    var request = require('request');

    module.exports.Lock = class Lock {
        constructor(name, lockAcquiredFn, lockLostFn) {
            this.name = name;
            this.baseUrl = 'http://localhost:13131';
            this.locked = false;
            this.lockAcquiredFn = lockAcquiredFn;
            this.lockLostFn = lockLostFn;
        }

        lock() {
            if (this.locked) {
                throw new Error('Locks are not reentrant');
            }
            this.locked = true;
            this.lockInternal();
        }

        lockInternal() {
            request(this.baseUrl + '/locks/' + this.name,
                (error, response) => {
                    // console.log('error:', error);
                    // console.log('statusCode:', response && response.statusCode);
                    var code = response.statusCode;
                    if (code == 404 || code == 200) {
                        this.updateLock(true);
                        return;
                    }
                    if (code != 200) {
                        // console.log('waiting for lock');
                        setTimeout(this.lockInternal.bind(this), 10 * 1000);
                    }
                });
        }

        updateLock(notify) {
            request.put(this.baseUrl + '/locks/' + this.name,
                (error, response) => {
                    // console.log('error:', error);
                    // console.log('statusCode:', response && response.statusCode);
                    var code = response.statusCode;
                    if (code == 200) {
                        if (notify && this.lockAcquiredFn) {
                            this.lockAcquiredFn();
                        }
                        setTimeout(this.holdLock.bind(this), 10 * 1000);
                    } else {
                        // console.log('waiting for lock');
                        setTimeout(this.lockInternal.bind(this), 10 * 1000);
                    }
                });
        }

        holdLock() {
            if (!this.locked) {
                return;
            }
            request(this.baseUrl + '/locks/' + this.name,
                (error, response) => {
                    // console.log('error:', error);
                    // console.log('statusCode:', response && response.statusCode);
                    var code = response.statusCode;
                    if (code == 200) {
                        this.updateLock(false);
                        return;
                    }
                    console.log('Unexpected code: ' + code);
                    if (this.lockLostFn) {
                        this.lockLostFn();
                    } else {
                        process.exit();
                    }
                });
        }

        unlock() {
            if (!this.locked) {
                throw new Error('Not locked!');
            }
            this.locked = false;
        }
    };
})();