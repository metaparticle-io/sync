(function() {
    var request = require('request');
    var log = require('loglevel');

    module.exports.debug = function() {
        log.setLevel("debug");
    }

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
                    log.debug('error:', error);
                    log.debug('statusCode:', response && response.statusCode);
                    if (error) {
                        if (error.code == 'ECONNREFUSED') {
                            log.error('Could not connect to ' + this.baseUrl + ' is the elector sidecar running?');
                        } else {
                            log.error('Unexpected error: ' + error);
                        }
                        process.exit(1);
                    }
                    var code = response.statusCode;
                    if (code == 404 || code == 200) {
                        this.updateLock(false);
                        return;
                    }
                    if (code != 200) {
                        // console.log('waiting for lock');
                        setTimeout(this.lockInternal.bind(this), 10 * 1000);
                    }
                });
        }

        updateLock(lockHeld) {
            request.put(this.baseUrl + '/locks/' + this.name,
                (error, response) => {
                    log.debug('error:', error);
                    log.debug('statusCode:', response && response.statusCode);
                    var code = response.statusCode;
                    if (code == 200) {
                        if (!lockHeld) {
                            this.lockAcquiredFn();
                        }
                        setTimeout(this.holdLock.bind(this), 10 * 1000);
                    } else {
                        if (lockHeld) {
                            log.warn('Unexpected code: ' + code);
                            if (this.lockLostFn) {
                                this.lockLostFn();
                            } else {
                                process.exit();
                            }
                        } else {
                            // console.log('waiting for lock');
                            setTimeout(this.lockInternal.bind(this), 10 * 1000);
                        }
                    }
                });
        }

        holdLock() {
            if (!this.locked) {
                return;
            }
            request(this.baseUrl + '/locks/' + this.name,
                (error, response) => {
                    log.debug('error:', error);
                    log.debug('statusCode:', response && response.statusCode);
                    var code = response.statusCode;
                    if (code == 200) {
                        this.updateLock(true);
                        return;
                    }
                    log.warn('unexpected code getting lock: ' + code);
                    setTimeout(this.holdLock.bind(this), 10 * 1000);
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