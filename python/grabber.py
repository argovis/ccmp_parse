# https://data.remss.com/ccmp/v03.0/daily/y1993/m06/

def paddit(i):
    if i>9:
        return str(i)
    else:
        return '0' + str(i)

for y in range(1993,2020):
    for m in range(1,13):
        for d in range(1,32):
            print(f'wget https://data.remss.com/ccmp/v03.0/daily/y{y}/m{paddit(m)}/CCMP_Wind_Analysis_{y}{paddit(m)}{paddit(d)}_V03.0_L4.0.nc -O CCMP_Wind_Analysis_{y}{paddit(m)}{paddit(d)}_V03.0_L4.0.nc')
