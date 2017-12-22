import metaparticle_sync
import threading
import time

def master_fn():
    print('I am the master')

def lost_master_fn():
    print('I lost the master')

el = metaparticle_sync.Election('test', master_fn, lost_master_fn)

def run():
    time.sleep(30)
    el.shutdown()

def main():
    t = threading.Thread(None, target=run)
    t.start()
    el.run()

if __name__ == "__main__":
    main()
