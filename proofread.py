import datetime, xarray, random, math

def timewindow(center, radius):
    # given a string specifying the central date in the format "1993-02-07T00:00:00.000Z",
    # produce a list of strings for the days +- radius around that date in the format yyyymmdd

    centerdatetime = datetime.datetime.strptime(center, "%Y-%m-%dT%H:%M:%SZ")
    datestrings = []
    for i in range(-1*radius, radius+1):
    	datestrings.append( (centerdatetime + datetime.timedelta(days=i)).strftime("%Y%m%d") )

    return datestrings

year = '1993'
month = '01'
day = '17'
dates = timewindow(f"{year}-{month}-{day}T00:00:00Z", 3)
print(dates)
timeidx = 1; # time index the timewindow date corresponds to
xars = [xarray.open_dataset(f"/tmp/y{year}/m{month}/CCMP_Wind_Analysis_{date}_V03.0_L4.0.nc", decode_times=False, mask_and_scale=False) for date in dates]
means = xarray.open_dataset(f"/tmp/ccmp_means_{year}.nc", decode_times=False)

while True:
	lat = math.floor(random.random()*720)
	lon = math.floor(random.random()*1440)

	total = 0
	nobs = 0
	for xar in xars:
		if not math.isnan(xar['vwnd'][lat][lon][0].to_dict()['data']) and not math.isnan(xar['vwnd'][lat][lon][1].to_dict()['data']) and not math.isnan(xar['vwnd'][lat][lon][2].to_dict()['data']) and not math.isnan(xar['vwnd'][lat][lon][3].to_dict()['data']):
			total += xar['vwnd'][lat][lon][0].to_dict()['data'] + xar['vwnd'][lat][lon][1].to_dict()['data'] + xar['vwnd'][lat][lon][2].to_dict()['data'] + xar['vwnd'][lat][lon][3].to_dict()['data']
			nobs += 4

	if means['vwnd'][timeidx][lat][lon].to_dict()['data'] != -999.9 and (not math.isclose(total/nobs, means['vwnd'][timeidx][lat][lon].to_dict()['data'], abs_tol=1e-5) or not math.isclose(nobs, means['vwnd_nobs'][timeidx][lat][lon].to_dict()['data'], abs_tol=1e-5)):
		print(total/nobs, means['vwnd'][timeidx][lat][lon].to_dict()['data'], nobs, means['vwnd_nobs'][timeidx][lat][lon].to_dict()['data'])