YEAR=1996
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m01
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m02
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m03
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m04
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m05
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m06
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m07
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m08
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m09
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m10
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m11
docker container run -e FTPUSER=William.Mills-1@colorado.edu -e FTPPASS=William.Mills-1@colorado.edu -v $(pwd)/data/ccmp:/tmp argovis/ccmp:dev python download-month.py $YEAR m12

