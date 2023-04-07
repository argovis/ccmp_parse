use netcdf;

fn main() -> Result<(), netcdf::error::Error> {

    let file = netcdf::open("data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc")?;

    let latitude = &file.variable("latitude").expect("Could not find variable 'latitude'");
    let longitude = &file.variable("longitude").expect("Could not find variable 'longitude'");
    let time = &file.variable("time").expect("Could not find variable 'time'");
    let vwnd = &file.variable("vwnd").expect("Could not find variable 'vwnd'");
 
    let data_f64 : f64 = latitude.value(719)?;    
    let vwnd_f64 = vwnd.value::<f64, _>([0, 0, 0])?;
    println!("latitude {}", data_f64);
    println!("vwnd {}", vwnd_f64);
    println!("lat len {}", latitude.len());

    for latidx in (0..latitude.len()) {
        for lonidx in (0..longitude.len()) {
            for timeidx in (0..time.len()) {
		println!("vwnd {} {} {} : {}", latidx, lonidx, timeidx, vwnd.value::<f64, _>([latidx, lonidx, timeidx])?)
            }
        }
    }

    return Ok(())

}
