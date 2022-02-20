use std::error::Error;
use clap::Parser;
use ndarray::{s,Array,Array1,Array2,Axis};
use ndarray_stats::CorrelationExt;
use ndarray_rand::{RandomExt,rand_distr::Normal};
use ndarray_linalg::{Solve,Cholesky,solveh::UPLO};

pub type Res<T> = Result<T,Box<dyn Error>>;

#[derive(Parser,Debug)]
#[clap(author,version,about,long_about=None)]
/// Perform a multinormality test
struct Cmd {
    /// Path to HDF5 file
    #[clap(long)]
    path:String,

    /// Name of variable to test
    #[clap(long)]
    name:String,

    /// Simulate
    #[clap(long)]
    simulate:bool,

    /// Print mean and covariance
    #[clap(long)]
    details:bool,

    /// Row range
    #[clap(long,number_of_values=2)]
    irange:Option<Vec<usize>>,

    /// Column range
    #[clap(long,number_of_values=2)]
    jrange:Option<Vec<usize>>
}

pub struct MardiaTest {
    x_mu:Array1<f64>,
    x_cov:Array2<f64>,
    a:f64,
    a_mu:f64,
    a_sigma:f64,
    a_z:f64,
    b:f64
}

impl MardiaTest {
    fn new(x:&Array2<f64>)->Res<Self> {
	let (m,n) = x.dim();
	let x_mu = x.mean_axis(Axis(0)).unwrap();
	let x_cov = x.t().cov(0.0)?;
	let mut s = 0.0;
	let mut k = 0.0;
	for i1 in 0..m {
	    let x1 = &x.slice(s![i1,..]) - &x_mu;
	    for i2 in i1..m {
		let x2 = &x.slice(s![i2,..]) - &x_mu;
		let y = x_cov.solve(&x2)?;
		let y = x1.dot(&y);
		let y3 = y.powi(3);
		if i2 > i1 {
		    s += 2.0 * y3;
		} else {
		    s += y3;
		    k += y.powi(2);
		}
	    }
	}
	let a = s / (6.0 * m as f64);
	let b = (m as f64 / (8*n*(n+2)) as f64).sqrt() *
	    (k / m as f64 - (n*(n + 2)) as f64);

	let a_mu = (n * (n + 1) * (n + 2)) as f64 / 6.0;
	let a_sigma = (2.0*a_mu).sqrt();
	let a_z = (a - a_mu) / a_sigma;

	Ok(Self{
	    x_mu,
	    x_cov,
	    a,
	    a_mu,
	    a_sigma,
	    a_z,
	    b
	})
    }
}

fn main()->Res<()> {
    hdf5::silence_errors(true);
    let args = Cmd::parse();
    let fd = hdf5::File::open(&args.path)?;
    let x_ds = fd.dataset(&args.name)?;
    let x : Array2<f64> = x_ds.read()?;
    let (m,n) = x.dim();
    println!("HDF5 path:    {}",args.path);
    println!("Dataset name: {}",args.name);
    println!("Dimensions:   {} by {}",m,n);

    let x =
	match args.irange {
	    None => x,
	    Some(rv) => {
		let i0 = rv[0];
		let i1 = rv[1];
		println!("Row range:    {} to {}",i0,i1);
		x.slice(s![i0..i1,..]).to_owned()
	    }
	};

    let x =
	match args.jrange {
	    None => x,
	    Some(rv) => {
		let j0 = rv[0];
		let j1 = rv[1];
		println!("Column range: {} to {}",j0,j1);
		x.slice(s![..,j0..j1]).to_owned()
	    }
	};

    let (m,n) = x.dim();

    let x = 
	if args.simulate {
	    let x_mu = x.mean_axis(Axis(0)).unwrap();
	    let x_cov = x.t().cov(0.0)?;
	    let h = x_cov.cholesky(UPLO::Upper)?;
	    let normal = Normal::new(0.0,1.0)?;
	    let u : Array2<f64> = Array::random((m,n),normal);
	    u.dot(&h) + &x_mu
	} else {
	    x
	};

    println!("Eff. dims.:   {} by {}",m,n);

    let mardia = MardiaTest::new(&x)?;
    println!("A : got {:.1}, expected {:.1} plus or minus {:.3}, Z-score {:.3}",
	     mardia.a,
	     mardia.a_mu,
	     mardia.a_sigma,
	     mardia.a_z);
    println!("B : got {:.3}, expected 0 plus or minus 1",
	     mardia.b);

    if args.details {
	println!("Mean:");
	for j in 0..n {
	    println!("  {:2} {}",j,mardia.x_mu[j]);
	}
	println!("Covariance: {:?}",mardia.x_cov.dim());
	for i in 0..n {
	    for j in i..n {
		println!("  {:2} {:2} {}",i,j,mardia.x_cov[[i,j]]);
	    }
	}
    }
    
    Ok(())
}
