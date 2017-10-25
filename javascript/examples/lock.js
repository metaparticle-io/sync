var mp = require('@metaparticle/sync');

var lock = new mp.Lock('test',
   () => {
       console.log('I have the lock!');
       console.log('Holding the lock for 45 seconds');
       setTimeout(() => {
           lock.unlock();
           console.log('Unlocked');
       }, 45 * 1000);
   });

if (process.argv.length > 2) {
    lock.baseUrl = process.argv[2];
}

console.log('Locking');
lock.lock();
