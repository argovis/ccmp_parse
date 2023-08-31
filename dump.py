import xarray, sys

lat = -78.375
lon = 0.125

xar = xarray.open_dataset(sys.argv[1], decode_times=False, mask_and_scale=False)
latitude = xar['latitude'].to_dict()['data']
longitude = xar['longitude'].to_dict()['data']
latidx = latitude.index(lat)
lonidx = longitude.index(lon)

print(xar['uwnd'][latidx][lonidx][:].to_dict()['data'])
print(xar['vwnd'][latidx][lonidx][:].to_dict()['data'])
print(xar['ws'][latidx][lonidx][:].to_dict()['data'])
print(xar['nobs'][latidx][lonidx][:].to_dict()['data'])



