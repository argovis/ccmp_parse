// batches of grid points arch

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

// impementing a foreign trait on a forein struct //////////
// per the advice in https://stackoverflow.com/questions/76277096/deconstructing-enums-in-rust/76277117#76277117

struct Wrapper{
    s: String
}

impl std::convert::TryFrom<netcdf::attribute::AttrValue> for Wrapper {
    type Error = &'static str;

    fn try_from(value: netcdf::attribute::AttrValue) -> Result<Self, Self::Error> {

        if let netcdf::attribute::AttrValue::Str(v) = value {
            Ok(Wrapper{s: String::from(v)} )
        } else {
            Err("nope")
        }
    }
}
////////////////////

fn find_basin(basins: &netcdf::Variable, longitude: f64, latitude: f64) -> i64 {    
    let lonplus = (longitude-0.5).ceil()+0.5;
    let lonminus = (longitude-0.5).floor()+0.5;
    let latplus = (latitude-0.5).ceil()+0.5;
    let latminus = (latitude-0.5).floor()+0.5;

    let lonplus_idx = (lonplus - -179.5) as usize;
    let lonminus_idx = (lonminus - -179.5) as usize;
    let latplus_idx = (latplus - -77.5) as usize;
    let latminus_idx = (latminus - -77.5) as usize;

    //println!("{} {} {} {} {}", longitude, lonminus, lonminus_idx, lonplus, lonplus_idx);

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
        Ok(idx) => idx,
        Err(e) => panic!("basin problems: {:?} {:#?}", e, closecorner_idx)
    }   
}

fn eq_with_nan_eq(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || (a == b)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // command line argument extraction
    let args: Vec<String> = env::args().collect();

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
    let ccmp = client.database("argo").collection("ccmp");
    let ccmp_meta = client.database("argo").collection("ccmpMeta");

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

    /////////////////////////////////////////////////////////////////////////////////

    // file opening
    let file = netcdf::open(&args[1])?; // data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc

    // basin lookup
    let basinfile = netcdf::open("data/basinmask_01.nc")?;
    let basins = &basinfile.variable("BASIN_TAG").expect("Could not find variable 'BASIN_TAG'");

    // metadata construction

    // all times recorded as hours since Jan 1 1987
    let t0 = Utc.with_ymd_and_hms(1987, 1, 1, 0, 0, 0).unwrap();

    // variable extraction
    let vwnd = &file.variable("vwnd").expect("Could not find variable 'vwnd'");
    let uwnd = &file.variable("uwnd").expect("Could not find variable 'uwnd'");
    let ws   = &file.variable("ws").expect("Could not find variable 'ws'");
    let nobs = &file.variable("nobs").expect("Could not find variable 'nobs'");


    let mut timeseries = Vec::new();
    for _k in 0..10000{
        let file = netcdf::open(&args[1])?; // data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc
        let time = &file.variable("time").expect("Could not find variable 'time'");
        for timeidx in 0..time.len() {
            timeseries.push(bson::DateTime::parse_rfc3339_str((t0 + Duration::hours(time.value::<i64, _>(timeidx)?)).to_rfc3339().replace("+00:00", "Z")).unwrap() );
        }
    }

    let metadata = CcmpMetadoc{
        _id: String::from("ccmp"),
        data_type: String::from("ccmp-wind"),
        data_info: (
            vec!(String::from("uwnd"), String::from("vwnd"), String::from("ws"), String::from("nobs")),
            vec!(String::from("units"), String::from("long_name")),
            vec!(
                vec!(Wrapper::try_from(uwnd.attribute("units").unwrap().value().unwrap()).unwrap().s,Wrapper::try_from(uwnd.attribute("long_name").unwrap().value().unwrap()).unwrap().s),
                vec!(Wrapper::try_from(vwnd.attribute("units").unwrap().value().unwrap()).unwrap().s,Wrapper::try_from(vwnd.attribute("long_name").unwrap().value().unwrap()).unwrap().s),
                vec!(Wrapper::try_from(ws.attribute("units").unwrap().value().unwrap()).unwrap().s,Wrapper::try_from(ws.attribute("long_name").unwrap().value().unwrap()).unwrap().s),
                vec!(String::from("null"),Wrapper::try_from(nobs.attribute("long_name").unwrap().value().unwrap()).unwrap().s)
            )
        ),
        date_updated_argovis: bson::DateTime::parse_rfc3339_str(nowstring()).unwrap(),
        timeseries: timeseries,
        source: vec!(
            Sourcedoc{
                source: vec!(String::from("CCMP Wind Vector Analysis Product")),
                url: String::from("https://data.remss.com/ccmp/")
            }
        )
    };
    let metadata_doc = bson::to_document(&metadata).unwrap();

    ccmp_meta.insert_one(metadata_doc.clone(), None).await?;

    // data doc: start by building matrix of measurement values for a single latitude and a batch of longitudes:
    for latidx in 300..301 { //latitude.len() {
        let mut vwndbatch = Vec::new();
        let mut uwndbatch = Vec::new();
        let mut wsbatch   = Vec::new();
        let mut nobsbatch = Vec::new();
        for _lonidx in 0..100 { // longitude.len() {
            vwndbatch.push(Vec::new());
            uwndbatch.push(Vec::new());
            wsbatch.push(Vec::new());
            nobsbatch.push(Vec::new());
        }

        // universal info we can get out of one file once
        let f = netcdf::open(&args[1])?; // data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc
        let latitude = &f.variable("latitude").expect("Could not find variable 'latitude'");
        let longitude = &f.variable("longitude").expect("Could not find variable 'longitude'");
        println!("{} {}", latitude.len(), longitude.len()); // 720 1440

        for _ii in 0..10000 {
            let file = netcdf::open(&args[1])?; // data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc
            let time = &file.variable("time").expect("Could not find variable 'time'");
            let vwnd = &file.variable("vwnd").expect("Could not find variable 'vwnd'");
            let uwnd = &file.variable("uwnd").expect("Could not find variable 'uwnd'");
            let ws   = &file.variable("ws").expect("Could not find variable 'ws'");
            let nobs = &file.variable("nobs").expect("Could not find variable 'nobs'");

            for lonidx in 0..100 { // longitude.len() {
                for timeidx in 0..time.len() {
                    if eq_with_nan_eq(vwnd.value::<f64, _>([latidx, lonidx, timeidx])?, NAN) && eq_with_nan_eq(uwnd.value::<f64, _>([latidx, lonidx, timeidx])?, NAN) && eq_with_nan_eq(ws.value::<f64, _>([latidx, lonidx, timeidx])?, NAN) && eq_with_nan_eq(nobs.value::<f64, _>([latidx, lonidx, timeidx])?, NAN){
                        // if no measurement contains anything for this timestep, no need to write it to the db
                        continue;
                    }
                    vwndbatch[lonidx].push(vwnd.value::<f64, _>([latidx, lonidx, timeidx])? );
                    uwndbatch[lonidx].push(uwnd.value::<f64, _>([latidx, lonidx, timeidx])? );
                    wsbatch[lonidx].push(ws.value::<f64, _>([latidx, lonidx, timeidx])? );
                    nobsbatch[lonidx].push(nobs.value::<f64, _>([latidx, lonidx, timeidx])? );
                }
            }
        }

        // construct all the json docs and dump to a file
        let mut docs = Vec::new();
        for lonidx in 0..100 { // longitude.len()
            if vwndbatch[lonidx].len() > 0 {
                let lat = latitude.value::<f64, _>(latidx)?;
                let lon = tidylon(longitude.value::<f64, _>(lonidx)?);
                let basin = find_basin(&basins, lon, lat);
                let id = [lon.to_string(), lat.to_string()].join("_");
                let data = doc!{
                    "_id": id,
                    "metadata": ["ccmp"],
                    "basin": basin,
                    "geolocation": {
                        "type": "Point",
                        "coordinates": [lat, lon]
                    },
                    "data": [vwndbatch[lonidx].clone(), uwndbatch[lonidx].clone(), wsbatch[lonidx].clone(), nobsbatch[lonidx].clone()]
                };
                docs.push(data);
            }
        }
        ccmp.insert_many(docs, None).await?;
    }

    Ok(())
}
