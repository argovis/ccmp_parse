# usage example: python download-month.py 1993
from ftplib import FTP
import sys, os

year = sys.argv[1]
targetdir = '/tmp/y'+year
try:
	os.mkdir(targetdir)
except:
	pass

ftp = FTP('ftp.remss.com', user=os.environ['FTPUSER'], passwd=os.environ['FTPPASS'])

for month in ['m06', 'm07', 'm08', 'm09', 'm10', 'm11', 'm12']:
	ftp.cwd('/ccmp/v03.0/daily/y'+year+'/'+month)
	try:
		os.mkdir(targetdir + '/' + month)
	except:
		pass
	files = ftp.nlst()
	for f in files:
		file = open(targetdir + '/' + month + '/' + f, 'wb')
		ftp.retrbinary('RETR ' + f, file.write)
		file.close()

ftp.quit()