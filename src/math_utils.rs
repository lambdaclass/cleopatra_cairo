use std::ops::Shr;

use crate::{bigint, vm::errors::vm_errors::VirtualMachineError};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{abs, FromPrimitive, Signed};

///Returns the integer square root of the nonnegative integer n.
///This is the floor of the exact square root of n.
///Unlike math.sqrt(), this function doesn't have rounding error issues.
pub fn isqrt(n: &BigInt) -> Result<BigInt, VirtualMachineError> {
    //n.shr(1) = n.div_floor(2)
    if n.is_negative() {
        return Err(VirtualMachineError::SqrtNegative(n.clone()));
    }
    let mut x = n.clone();
    let mut y = (x.clone() + bigint!(1)).shr(1_i32);
    while y < x {
        x = y;
        y = (x.clone() + n.div_floor(&x)).shr(1_i32);
    }
    if !(x.pow(2) <= *n && *n < (x.clone() + bigint!(1)).pow(2)) {
        return Err(VirtualMachineError::FailedToGetSqrt(n.clone()));
    };
    Ok(x)
}

/// Returns the lift of the given field element, val, as an integer in the range (-prime/2, prime/2).
pub fn as_int(val: &BigInt, prime: &BigInt) -> BigInt {
    //n.shr(1) = n.div_floor(2)
    if *val < prime.shr(1) {
        val.clone()
    } else {
        val - prime
    }
}

///Returns x, y, g such that g = x*a + y*b = gcd(a, b).
fn igcdex(num_a: BigInt, num_b: BigInt) -> (BigInt, BigInt, BigInt) {
    let mut a = num_a;
    let mut b = num_b;
    let x_sign: i32;
    let y_sign: i32;
    match (a.clone(), b.clone()) {
        (a, b) if a == b && a == bigint!(0) => (bigint!(0), bigint!(1), bigint!(0)),
        (a, _) if a == bigint!(0) => (bigint!(0), b.div_floor(&abs(b.clone())), abs(b)),
        (_, b) if b == bigint!(0) => (a.div_floor(&a), bigint!(0), abs(a)),
        _ => {
            if a < bigint!(0) {
                a = -a;
                x_sign = -1;
            } else {
                x_sign = 1;
            }
            if b < bigint!(0) {
                b = -b;
                y_sign = -1;
            } else {
                y_sign = 1;
            }
            let (mut x, mut y, mut r, mut s) = (bigint!(1), bigint!(0), bigint!(0), bigint!(1));
            let (mut c, mut q);
            while b != bigint!(0) {
                (c, q) = (a.clone() % b.clone(), a.div_floor(&b.clone()));
                (a, b, r, s, x, y) = (b, c, x - q.clone() * r.clone(), y - q * s.clone(), r, s)
            }
            (x * x_sign, y * y_sign, a)
        }
    }
}
///Finds a nonnegative integer x < p such that (m * x) % p == n.
fn div_mod(n: BigInt, m: BigInt, p: BigInt) -> BigInt {
    let (a, _, c) = igcdex(m, p.clone());
    assert_eq!(c, bigint!(1));
    (n * a).mod_floor(&p)
}

/// Gets two points on an elliptic curve mod p and returns their sum.
/// Assumes the points are given in affine form (x, y) and have different x coordinates.
pub fn ec_add(
    point_a: (BigInt, BigInt),
    point_b: (BigInt, BigInt),
    prime: &BigInt,
) -> (BigInt, BigInt) {
    let m = line_slope(point_a.clone(), point_b.clone(), prime);
    let x = (m.clone() * m.clone() - point_a.0.clone() - point_b.0).mod_floor(prime);
    let y = (m * (point_a.0 - x.clone()) - point_a.1).mod_floor(prime);
    (x, y)
}

/// Computes the slope of the line connecting the two given EC points over the field GF(p).
/// Assumes the points are given in affine form (x, y) and have different x coordinates.
pub fn line_slope(point_a: (BigInt, BigInt), point_b: (BigInt, BigInt), prime: &BigInt) -> BigInt {
    assert!((point_a.0.clone() - point_b.0.clone()) % prime != bigint!(0));
    div_mod(point_a.1 - point_b.1, point_a.0 - point_b.0, prime.clone())
}

///  Doubles a point on an elliptic curve with the equation y^2 = x^3 + alpha*x + beta mod p.
/// Assumes the point is given in affine form (x, y) and has y != 0.
pub fn ec_double(point: (BigInt, BigInt), alpha: &BigInt, prime: &BigInt) -> (BigInt, BigInt) {
    let m = ec_double_slope(point.clone(), alpha, prime);
    let x = ((m.clone() * m.clone()) - (bigint!(2) * point.0.clone())) % prime;
    let y = (m * (point.0.clone() - x.clone()) - point.1).mod_floor(prime);
    (x, y)
}

/// Computes the slope of an elliptic curve with the equation y^2 = x^3 + alpha*x + beta mod p, at
/// the given point.
/// Assumes the point is given in affine form (x, y) and has y != 0.
pub fn ec_double_slope(point: (BigInt, BigInt), alpha: &BigInt, prime: &BigInt) -> BigInt {
    assert!(point.1.clone() % prime != bigint!(0));
    div_mod(
        bigint!(3) * point.0.clone() * point.0.clone() + alpha,
        bigint!(2) * point.1,
        prime.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bigint_str;

    #[test]
    fn calculate_igcdex() {
        let a = bigint_str!(
            b"3443173965374276972000139705137775968422921151703548011275075734291405722262"
        );
        let b = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!((bigint_str!(b"-1688547300931946713657663208540757607205184050780245505361433670721217394901"), bigint_str!(b"1606731415015725997151049087601104361134423282856790368548943305828633315023"), bigint!(1)), igcdex(a, b));
    }

    #[test]
    fn calculate_divmod_a() {
        let a = bigint_str!(
            b"11260647941622813594563746375280766662237311019551239924981511729608487775604310196863705127454617186486639011517352066501847110680463498585797912894788"
        );
        let b = bigint_str!(
            b"4020711254448367604954374443741161860304516084891705811279711044808359405970"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            bigint_str!(
                b"2904750555256547440469454488220756360634457312540595732507835416669695939476"
            ),
            div_mod(a, b, prime)
        );
    }

    #[test]
    fn calculate_divmod_b() {
        let a = bigint_str!(
            b"29642372811668969595956851264770043260610851505766181624574941701711520154703788233010819515917136995474951116158286220089597404329949295479559895970988"
        );
        let b = bigint_str!(
            b"3443173965374276972000139705137775968422921151703548011275075734291405722262"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            bigint_str!(
                b"3601388548860259779932034493250169083811722919049731683411013070523752439691"
            ),
            div_mod(a, b, prime)
        );
    }

    #[test]
    fn calculate_divmod_c() {
        let a = bigint_str!(
            b"1208267356464811040667664150251401430616174694388968865551115897173431833224432165394286799069453655049199580362994484548890574931604445970825506916876"
        );
        let b = bigint_str!(
            b"1809792356889571967986805709823554331258072667897598829955472663737669990418"
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            bigint_str!(
                b"1545825591488572374291664030703937603499513742109806697511239542787093258962"
            ),
            div_mod(a, b, prime)
        );
    }

    #[test]
    fn compute_line_slope_for_valid_points() {
        let point_a = (
            bigint_str!(
                b"3139037544796708144595053687182055617920475701120786241351436619796497072089"
            ),
            bigint_str!(
                b"2119589567875935397690285099786081818522144748339117565577200220779667999801"
            ),
        );
        let point_b = (
            bigint_str!(
                b"3324833730090626974525872402899302150520188025637965566623476530814354734325"
            ),
            bigint_str!(
                b"3147007486456030910661996439995670279305852583596209647900952752170983517249"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            bigint_str!(
                b"992545364708437554384321881954558327331693627531977596999212637460266617010"
            ),
            line_slope(point_a, point_b, &prime)
        );
    }

    #[test]
    fn compute_double_slope_for_valid_point_a() {
        let point = (
            bigint_str!(
                b"3143372541908290873737380228370996772020829254218248561772745122290262847573"
            ),
            bigint_str!(
                b"1721586982687138486000069852568887984211460575851774005637537867145702861131"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let alpha = bigint!(1);
        assert_eq!(
            bigint_str!(
                b"3601388548860259779932034493250169083811722919049731683411013070523752439691"
            ),
            ec_double_slope(point, &alpha, &prime)
        );
    }

    #[test]
    fn compute_double_slope_for_valid_point_b() {
        let point = (
            bigint_str!(
                b"1937407885261715145522756206040455121546447384489085099828343908348117672673"
            ),
            bigint_str!(
                b"2010355627224183802477187221870580930152258042445852905639855522404179702985"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let alpha = bigint!(1);
        assert_eq!(
            bigint_str!(
                b"2904750555256547440469454488220756360634457312540595732507835416669695939476"
            ),
            ec_double_slope(point, &alpha, &prime)
        );
    }

    #[test]
    fn calculate_ec_double_for_valid_point_a() {
        let point = (
            bigint_str!(
                b"1937407885261715145522756206040455121546447384489085099828343908348117672673"
            ),
            bigint_str!(
                b"2010355627224183802477187221870580930152258042445852905639855522404179702985"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let alpha = bigint!(1);
        assert_eq!(
            (
                bigint_str!(
                    b"58460926014232092148191979591712815229424797874927791614218178721848875644"
                ),
                bigint_str!(
                    b"1065613861227134732854284722490492186040898336012372352512913425790457998694"
                )
            ),
            ec_double(point, &alpha, &prime)
        );
    }

    #[test]
    fn calculate_ec_double_for_valid_point_b() {
        let point = (
            bigint_str!(
                b"3143372541908290873737380228370996772020829254218248561772745122290262847573"
            ),
            bigint_str!(
                b"1721586982687138486000069852568887984211460575851774005637537867145702861131"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let alpha = bigint!(1);
        assert_eq!(
            (
                bigint_str!(
                    b"1937407885261715145522756206040455121546447384489085099828343908348117672673"
                ),
                bigint_str!(
                    b"2010355627224183802477187221870580930152258042445852905639855522404179702985"
                )
            ),
            ec_double(point, &alpha, &prime)
        );
    }

    #[test]
    fn calculate_ec_double_for_valid_point_c() {
        let point = (
            bigint_str!(
                b"634630432210960355305430036410971013200846091773294855689580772209984122075"
            ),
            bigint_str!(
                b"904896178444785983993402854911777165629036333948799414977736331868834995209"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        let alpha = bigint!(1);
        assert_eq!(
            (
                bigint_str!(
                    b"3143372541908290873737380228370996772020829254218248561772745122290262847573"
                ),
                bigint_str!(
                    b"1721586982687138486000069852568887984211460575851774005637537867145702861131"
                )
            ),
            ec_double(point, &alpha, &prime)
        );
    }

    #[test]
    fn calculate_ec_add_for_valid_points_a() {
        let point_a = (
            bigint_str!(
                b"1183418161532233795704555250127335895546712857142554564893196731153957537489"
            ),
            bigint_str!(
                b"1938007580204102038458825306058547644691739966277761828724036384003180924526"
            ),
        );
        let point_b = (
            bigint_str!(
                b"1977703130303461992863803129734853218488251484396280000763960303272760326570"
            ),
            bigint_str!(
                b"2565191853811572867032277464238286011368568368717965689023024980325333517459"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            (
                bigint_str!(
                    b"1977874238339000383330315148209250828062304908491266318460063803060754089297"
                ),
                bigint_str!(
                    b"2969386888251099938335087541720168257053975603483053253007176033556822156706"
                )
            ),
            ec_add(point_a, point_b, &prime)
        );
    }

    #[test]
    fn calculate_ec_add_for_valid_points_b() {
        let point_a = (
            bigint_str!(
                b"3139037544796708144595053687182055617920475701120786241351436619796497072089"
            ),
            bigint_str!(
                b"2119589567875935397690285099786081818522144748339117565577200220779667999801"
            ),
        );
        let point_b = (
            bigint_str!(
                b"3324833730090626974525872402899302150520188025637965566623476530814354734325"
            ),
            bigint_str!(
                b"3147007486456030910661996439995670279305852583596209647900952752170983517249"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            (
                bigint_str!(
                    b"1183418161532233795704555250127335895546712857142554564893196731153957537489"
                ),
                bigint_str!(
                    b"1938007580204102038458825306058547644691739966277761828724036384003180924526"
                )
            ),
            ec_add(point_a, point_b, &prime)
        );
    }

    #[test]
    fn calculate_ec_add_for_valid_points_c() {
        let point_a = (
            bigint_str!(
                b"1183418161532233795704555250127335895546712857142554564893196731153957537489"
            ),
            bigint_str!(
                b"1938007580204102038458825306058547644691739966277761828724036384003180924526"
            ),
        );
        let point_b = (
            bigint_str!(
                b"1977703130303461992863803129734853218488251484396280000763960303272760326570"
            ),
            bigint_str!(
                b"2565191853811572867032277464238286011368568368717965689023024980325333517459"
            ),
        );
        let prime = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(
            (
                bigint_str!(
                    b"1977874238339000383330315148209250828062304908491266318460063803060754089297"
                ),
                bigint_str!(
                    b"2969386888251099938335087541720168257053975603483053253007176033556822156706"
                )
            ),
            ec_add(point_a, point_b, &prime)
        );
    }

    #[test]
    fn calculate_isqrt_a() {
        let n = bigint!(81);
        assert_eq!(isqrt(&n), Ok(bigint!(9)));
    }

    #[test]
    fn calculate_isqrt_b() {
        let n = bigint_str!(b"4573659632505831259480");
        assert_eq!(isqrt(&(n.pow(2))), Ok(n));
    }

    #[test]
    fn calculate_isqrt_c() {
        let n = bigint_str!(
            b"3618502788666131213697322783095070105623107215331596699973092056135872020481"
        );
        assert_eq!(isqrt(&(n.pow(2))), Ok(n));
    }

    #[test]
    fn calculate_isqrt_zero() {
        let n = bigint!(0);
        assert_eq!(isqrt(&n), Ok(bigint!(0)));
    }

    #[test]
    fn calculate_isqrt_negative() {
        let n = bigint!(-1);
        assert_eq!(isqrt(&n), Err(VirtualMachineError::SqrtNegative(n)));
    }
}
