# CCMP parsing

This branch is for taking upstream CCMP data and reducing it to weekly averages in an intermediate set of netcdf files; see branch `db-population` for writing these weekly averages to mongodb.

## Downloading data

 - this Argovis product based on the 6h wind measurements at https://www.remss.com/measurements/ccmp/ 
 - build and push container image described in `Dockerfile-ftp` as `argovis/ccmp:download`
 - put REMSS FTP credentials in `pod-template.yaml`
 - edit and run `dl.sh` to download data as desired.
 - a large openshift volume named `ccmp` is expected to be available.

## Computing weekly averages

 - weekly averages aligned to match https://psl.noaa.gov/data/gridded/data.noaa.oisst.v2.html
 - build and push image described in `Dockerfile` as `argovis/ccmp:reduce` and run via `pod-reduce.yaml` after choosing the appropriate year. Resource intensive, takes about 14h on openshift.

## Proofreading

 - build and push image described in `Dockerfile-proofread` as `argovis/ccmp:reduce-proofread` and run via `pod-prrofread.yaml`
 - Randomly checks for consistency between upstream and reduced data until interrupted.
 - Use `dump.py` from the same container to manually inspect the upstream values for a given lat/lon in a given upstream file.