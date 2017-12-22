from metaparticle_sync import Lock
import time

def main():
	l = Lock('test')
	l.acquire()
	print('I have the lock!')
	time.sleep(30)
	l.release()

if __name__ == "__main__":
	main()
