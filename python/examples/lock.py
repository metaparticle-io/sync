from metaparticle_sync import Lock
import time

l = Lock('test')
l.acquire()
print('I have the lock!')
time.sleep(30)
l.release()
