var mp = require('@metaparticle/sync');

var election = new mp.Election(
    'test',
    () => {
        console.log('I am the master');
    },
    () => {
        console.log('I lost the master');
    });
election.run();
