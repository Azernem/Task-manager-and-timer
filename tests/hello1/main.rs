fn enter() { 
    println!("Hello, world!"); 
} 
fn add(a: u32, b: u32) -> u32 { 
    a + b 
} 
#[cfg(test)] 
mod tests 
{ 
    use super::*; 
    #[test] 
    fn test_adding() 
    { 
        let res: u32 = add(2, 4); 
        assert_eq!(res,6); 
    } 
 
}