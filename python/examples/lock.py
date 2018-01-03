import time
from metaparticle_sync import Lock


def main():
    lock = Lock('test')
    lock.acquire()
    print('I have the lock!')
    time.sleep(30)
    lock.release()


if __name__ == "__main__":
    main()
