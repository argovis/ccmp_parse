# CCMP parsing

This branch is for taking upstream CCMP data and reducing it to weekly averages in an intermediate set of netcdf files; see branch `db-population` for writing these weekly averages to mongodb.

## Downloading data

 - this Argovis product based on the 6h wind measurements at https://www.remss.com/measurements/ccmp/ 
 - build container image described in `Dockerfile-ftp` as `argovis/ccmp:dev`
 - edit and run `dl.sh` to download a year of data from CCMP. Note connection to their FTP servers tends to be fraught.
 - use `download-day.py` from the same container to re-try any individual days that failed in the yearly dl.

## Computing weekly averages

 - weekly averages aligned to match https://psl.noaa.gov/data/gridded/data.noaa.oisst.v2.html
 - build image described in `Dockerfile` and run as for example `docker container run -v $(pwd)/data/ccmp:/tmp argovis/ccmp:reduce cargo run 1993`; note the mount should place the data downloaded above at `/tmp/y1993` for example.

## Proofreading

 - build image described in `Dockerfile-proofread` and run as `docker container run -v $(pwd)/data/ccmp:/tmp argovis/ccmp:proofread`, same data mount as above.
 - Randomly checks for consistency between upstream and reduced data until interrupted.
 - Use `dump.py` from the same container to manually inspect the upstream values for a given lat/lon in a given upstream file.