# usage example: python download-day.py 1993 10 16
from ftplib import FTP
import sys, os

year = sys.argv[1]
month = sys.argv[2]
day = sys.argv[3]
targetdir = '/tmp/y'+year + '/m' + month
targetfile = f'CCMP_Wind_Analysis_{year}{month}{day}_V03.0_L4.0.nc' 

ftp = FTP('ftp.remss.com', user=os.environ['FTPUSER'], passwd=os.environ['FTPPASS'])
ftp.cwd('/ccmp/v03.0/daily/y'+year+'/m'+month)
file = open(targetdir + '/' + targetfile, 'wb')
ftp.retrbinary('RETR ' + targetfile, file.write)
ftp.quit()
