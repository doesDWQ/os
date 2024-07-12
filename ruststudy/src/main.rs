

pub struct PhysAddr(usize);
pub struct PhysPageNum(String);

impl From<PhysAddr> for PhysPageNum {
    fn from(v: PhysAddr) -> Self {
        Self(String::from("1页"))
    }
}

impl From<PhysPageNum> for  PhysAddr {
    fn from(value: PhysPageNum) -> Self {
        Self(1)
    }
}

fn main() {
   let physAddr = PhysAddr(1);
   let x: PhysPageNum = physAddr.into();
   println!("{}",x.0);

   let x2 = PhysAddr::from(x);
   println!("{}",x2.0);
}
