use netcdf;
use chrono::Utc;
use chrono::TimeZone;
use chrono::Duration;
use chrono::Datelike;
use chrono::Timelike;
use std::env;
use std::f64::NAN;
use std::error::Error;
use mongodb::{Client, options::{ClientOptions, ResolverConfig}};
use tokio;
use mongodb::bson::{doc};
use serde::{Deserialize, Serialize};
use mongodb::bson::DateTime;

fn tidylon(longitude: f64) -> f64{
    // map longitude on [0,360] to [-180,180], required for mongo indexing
    if longitude <= 180.0{
        return longitude;
    }
    else{
        return longitude-360.0;
    }
}

fn nowstring() -> String{
    // returns a String representing the current ISO8601 datetime

    let now = Utc::now();
    return format!("{}-{:02}-{:02}T{:02}:{:02}:{:02}Z", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
}

fn find_basin(basins: &netcdf::Variable, longitude: f64, latitude: f64) -> i32 {    
    let lonplus = (longitude-0.5).ceil()+0.5;
    let lonminus = (longitude-0.5).floor()+0.5;
    let latplus = (latitude-0.5).ceil()+0.5;
    let latminus = (latitude-0.5).floor()+0.5;

    let lonplus_idx = (lonplus - -179.5) as usize;
    let lonminus_idx = (lonminus - -179.5) as usize;
    let latplus_idx = (latplus - -77.5) as usize;
    let latminus_idx = (latminus - -77.5) as usize;

    let corners_idx = [
        // bottom left corner, clockwise
        [latminus_idx, lonminus_idx],
        [latplus_idx, lonminus_idx],
        [latplus_idx, lonplus_idx],
        [latminus_idx, lonplus_idx]
    ];

    let distances = [
        (f64::powi(longitude-lonminus, 2) + f64::powi(latitude-latminus, 2)).sqrt(),
        (f64::powi(longitude-lonminus, 2) + f64::powi(latitude-latplus, 2)).sqrt(),
        (f64::powi(longitude-lonplus, 2) + f64::powi(latitude-latplus, 2)).sqrt(),
        (f64::powi(longitude-lonplus, 2) + f64::powi(latitude-latminus, 2)).sqrt()
    ];

    let mut closecorner_idx = corners_idx[0];
    let mut closedist = distances[0];
    for i in 1..4 {
        if distances[i] < closedist{
            closecorner_idx = corners_idx[i];
            closedist = distances[i];
        }
    }

    match basins.value::<i64,_>(closecorner_idx){
        Ok(idx) => idx as i32,
        Err(e) => panic!("basin problems: {:?} {:#?}", e, closecorner_idx)
    }   
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // fixed coordinates
    
    let batchfiles = ["/bulk/ccmp/ccmp_means_1993.nc"];

    // mongodb setup ////////////////////////////////////////////////////////////
    // Load the MongoDB connection string from an environment variable:
    let client_uri =
       env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!"); 

    // A Client is needed to connect to MongoDB:
    // An extra line of code to work around a DNS issue on Windows:
    let options =
       ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
          .await?;
    let client = Client::with_options(options)?; 

    // collection objects
    let ccmp = client.database("argo").collection("ccmpwind");
    let ccmp_meta = client.database("argo").collection("timeseriesMeta");
    let summaries = client.database("argo").collection("summaries");

    // Rust structs to serialize time properly
    #[derive(Serialize, Deserialize, Debug)]
    struct Sourcedoc {
        source: Vec<String>,
        url: String
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CcmpMetadoc {
        _id: String,
        data_type: String,
        data_info: (Vec<String>, Vec<String>, Vec<Vec<String>>),
        date_updated_argovis: DateTime,
        timeseries: Vec<DateTime>,
        source: Vec<Sourcedoc>
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct summaryDoc {
        _id: String,
        data: Vec<String>,
        longitude_grid_spacing_degrees: f64,
        latitude_grid_spacing_degrees: f64,
        longitude_center: f64,
        latitude_center: f64
    }

    /////////////////////////////////////////////////////////////////////////////////

    // metadata construction

    // all times recorded as days since Jan 1 1993
    let t0 = Utc.with_ymd_and_hms(1993, 1, 1, 0, 0, 0).unwrap();

    let mut timeseries = Vec::new();
    for _k in 0..batchfiles.len(){
        let file = netcdf::open(batchfiles[_k])?;
        let timestamps = &file.variable("timestamps").expect("Could not find variable 'timestamps'");
        for timeidx in 0..timestamps.len() {
            timeseries.push(bson::DateTime::parse_rfc3339_str((t0 + Duration::days(timestamps.value::<i64, _>(timeidx)?)).to_rfc3339().replace("+00:00", "Z")).unwrap() );
        }
    }

    let metadata = CcmpMetadoc{
        _id: String::from("ccmpwind"),
        data_type: String::from("wind vector"),
        data_info: (
            vec!(String::from("uwnd"),String::from("vwnd"),String::from("ws"),String::from("nobs")),
            vec!(String::from("units"), String::from("long_name")),
            vec!(
                vec!(String::from("m s-1"), String::from("u-wind vector component at 10 meters, averaged weekly")),
                vec!(String::from("m s-1"), String::from("v-wind vector component at 10 meters, averaged weekly")),
                vec!(String::from("m s-1"), String::from("wind speed at 10 meters, averaged weekly")),
                vec!(String::from(""), String::from("number of observations used to derive wind vector components, averaged weekly"))
            )
        ),
        date_updated_argovis: bson::DateTime::parse_rfc3339_str(nowstring()).unwrap(),
        timeseries: timeseries,
        source: vec!(
            Sourcedoc{
                source: vec!(String::from("REMSS CCMP wind vector analysis product")),
                url: String::from("https://www.remss.com/measurements/ccmp/")
            }
        )
    };
    let metadata_doc = bson::to_document(&metadata).unwrap();
    ccmp_meta.insert_one(metadata_doc.clone(), None).await?;

    // construct summary doc
    let summary = summaryDoc {
        _id: String::from("ccmpwindsummary"),
        data: vec!(String::from("uwnd"),String::from("vwnd"),String::from("ws"),String::from("nobs")),
        longitude_grid_spacing_degrees: 0.25,
        latitude_grid_spacing_degrees: 0.25,
        longitude_center: 0.125,
        latitude_center: 0.125
    };
    let summary_doc = bson::to_document(&summary).unwrap();
    summaries.insert_one(summary_doc.clone(), None).await?;

    // data doc: start by building matrix of measurement values for a single latitude and all the longitudes:

    // basin lookup
    let basinfile = netcdf::open("/bulk/ccmp/basinmask_01.nc")?;
    let basins = &basinfile.variable("BASIN_TAG").expect("Could not find variable 'BASIN_TAG'");

    for latidx in 0..720 {
        println!("latindex {}", latidx);
        let mut meanuwndbatch = Vec::new();
        let mut meanvwndbatch = Vec::new();
        let mut meanwsbatch = Vec::new();
        let mut meannobsbatch = Vec::new();
        for _lonidx in 0..1440 {
            meanuwndbatch.push(Vec::new());
            meanvwndbatch.push(Vec::new());
            meanwsbatch.push(Vec::new());
            meannobsbatch.push(Vec::new());
        }

        for _f in 0..batchfiles.len() { // ie for every year
            let file = netcdf::open(batchfiles[_f])?; 
            let uwnd = &file.variable("uwnd").expect("Could not find variable 'uwnd'");
            let vwnd = &file.variable("vwnd").expect("Could not find variable 'vwnd'");
            let ws   = &file.variable("ws").expect("Could not find variable 'ws'");
            let nobs = &file.variable("nobs").expect("Could not find variable 'nobs'");
            let uwnd_nobs = &file.variable("uwnd_nobs").expect("Could not find variable 'uwnd_nobs'");
            let vwnd_nobs = &file.variable("vwnd_nobs").expect("Could not find variable 'vwnd_nobs'");
            let ws_nobs   = &file.variable("ws_nobs").expect("Could not find variable 'ws_nobs'");
            let timestamps = &file.variable("timestamps").expect("Could not find variable 'timestamps'");

            for lonidx in 0..1440 {
                for timeidx in 0..timestamps.len() {
                    let uwnd_v = uwnd.value::<f64, _>([timeidx, latidx, lonidx])?;
                    let uwnd_n = uwnd_nobs.value::<i64, _>([timeidx, latidx, lonidx])?;
                    if uwnd_v != -999.9 && uwnd_n == 28 { // ie mask out means that didnt have all 28 6h periods available
                        meanuwndbatch[lonidx].push(uwnd_v);
                    } else {
                        meanuwndbatch[lonidx].push(NAN);
                    }

                    let vwnd_v = vwnd.value::<f64, _>([timeidx, latidx, lonidx])?;
                    let vwnd_n = vwnd_nobs.value::<i64, _>([timeidx, latidx, lonidx])?;
                    if vwnd_v != -999.9 && vwnd_n == 28 { // ie mask out means that didnt have all 28 6h periods available
                        meanvwndbatch[lonidx].push(vwnd_v);
                    } else {
                        meanvwndbatch[lonidx].push(NAN);
                    }

                    let ws_v = ws.value::<f64, _>([timeidx, latidx, lonidx])?;
                    let ws_n = ws_nobs.value::<i64, _>([timeidx, latidx, lonidx])?;
                    if ws_v != -999.9 && ws_n == 28 { // ie mask out means that didnt have all 28 6h periods available
                        meanwsbatch[lonidx].push(ws_v);
                    } else {
                        meanwsbatch[lonidx].push(NAN);
                    }

                    let nobs_v = nobs.value::<f64, _>([timeidx, latidx, lonidx])?;
                    if nobs_v != -999.9 { // ie mask out means that didnt have all 28 6h periods available
                        meannobsbatch[lonidx].push(nobs_v);
                    } else {
                        meannobsbatch[lonidx].push(NAN);
                    }

                }
            }
        }

        // construct all the json docs and dump to a file
        let mut docs = Vec::new();
        let file = netcdf::open(batchfiles[0])?;
        let latitude = &file.variable("latitude").expect("Could not find variable 'latitude'");
        let longitude = &file.variable("longitude").expect("Could not find variable 'longitude'");
        for lonidx in 0..1440 {
            let d_uwnd = meanuwndbatch[lonidx].clone();
            /// bail out if the whole timeseries is nan
            let mut nonnans_uwnd = 0;
            for _i in 0..d_uwnd.len() {
                if !d_uwnd[_i].is_nan() {
                    nonnans_uwnd += 1;
                }
            }
            if nonnans_uwnd == 0 {
                continue;
            }

            let d_vwnd = meanvwndbatch[lonidx].clone();
            /// bail out if the whole timeseries is nan
            let mut nonnans_vwnd = 0;
            for _i in 0..d_vwnd.len() {
                if !d_vwnd[_i].is_nan() {
                    nonnans_vwnd += 1;
                }
            }
            if nonnans_vwnd == 0 {
                continue;
            }

            let d_ws = meanwsbatch[lonidx].clone();
            /// bail out if the whole timeseries is nan
            let mut nonnans_ws = 0;
            for _i in 0..d_ws.len() {
                if !d_ws[_i].is_nan() {
                    nonnans_ws += 1;
                }
            }
            if nonnans_ws == 0 {
                continue;
            }

            let d_nobs = meannobsbatch[lonidx].clone();
            /// bail out if the whole timeseries is nan
            let mut nonnans_nobs = 0;
            for _i in 0..d_nobs.len() {
                if !d_nobs[_i].is_nan() {
                    nonnans_nobs += 1;
                }
            }
            if nonnans_nobs == 0 {
                continue;
            }

            let lat = latitude.value::<f64, _>([latidx])?;
            let lon = tidylon(longitude.value::<f64, _>([lonidx])?);
            let basin = find_basin(&basins, lon, lat);
            let id = [lon.to_string(), lat.to_string()].join("_");
            let data = doc!{
                "_id": id,
                "metadata": ["ccmpwind"],
                "basin": basin,
                "geolocation": {
                    "type": "Point",
                    "coordinates": [lon, lat]
                },
                "data": [d_uwnd, d_vwnd, d_ws, d_nobs]
            };
            docs.push(data);
        }
        if docs.len() > 0 {
            ccmp.insert_many(docs, None).await?;
        }
    }

    Ok(())
}
