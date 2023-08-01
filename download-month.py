# usage example: python download-month.py 1993 m01
from ftplib import FTP
import sys, os

year = sys.argv[1]
month = sys.argv[2]
targetdir = '/tmp/y'+year
try:
	os.mkdir(targetdir)
except:
	pass

try:
	os.mkdir(targetdir + '/' + month)
except:
	pass

ftp = FTP('ftp.remss.com', user=os.environ['FTPUSER'], passwd=os.environ['FTPPASS'])
ftp.cwd('/ccmp/v03.0/daily/y'+year+'/'+month)
files = ftp.nlst()
ftp.quit()

for f in files:
	# open / close ftp connection to try and avoid constant timeouts
	try:
		ftp = FTP('ftp.remss.com', user=os.environ['FTPUSER'], passwd=os.environ['FTPPASS'])
		ftp.cwd('/ccmp/v03.0/daily/y'+year+'/'+month)
		file = open(targetdir + '/' + month + '/' + f, 'wb')
		ftp.retrbinary('RETR ' + f, file.write)
		file.close()
		ftp.quit()
	except:
		print('failed ' + f)
