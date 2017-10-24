var mp = require('@metaparticle/sync');

var lock = new mp.Lock('test');
console.log('Locking');
lock.lock();
console.log('Waiting');
setTimeout(() => {
    lock.unlock();
    console.log('unlocked');
}, 45 * 1000);
