use netcdf;

fn main() -> Result<(), netcdf::error::Error> {

    let file = netcdf::open("data/CCMP_Wind_Analysis_19930103_V03.0_L4.0.nc")?;
    let var = &file.variable("latitude").expect("Could not find variable 'data'");
    let data_f32 : f32 = var.value(0)?;    

    println!("latitude {}", data_f32);

    return Ok(())

}
