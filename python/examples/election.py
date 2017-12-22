import metaparticle_sync

def master_fn():
    print('I am the master')

def lost_master_fn():
    print('I lost the master')

el = metaparticle_sync.Election('test', master_fn, lost_master_fn)
el.run()
