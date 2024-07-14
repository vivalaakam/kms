use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::Zero;
use num_traits::{Num, One};
use serde::{Deserialize, Serialize};

use crate::helpers::generate_code::generate_random;

lazy_static! {
    static ref PRIME: BigUint = BigUint::from_str_radix(
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",
        16
    )
    .expect("N parse error");
}

#[derive(Clone, Debug)]
pub struct Share {
    pub x: BigUint,
    pub y: BigUint,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShareStore {
    pub x: String,
    pub y: String,
}

impl From<Share> for ShareStore {
    fn from(share: Share) -> Self {
        ShareStore {
            x: hex::encode(share.x.to_bytes_be()),
            y: hex::encode(share.y.to_bytes_be()),
        }
    }
}

impl From<&Share> for ShareStore {
    fn from(share: &Share) -> Self {
        ShareStore {
            x: hex::encode(share.x.to_bytes_be()),
            y: hex::encode(share.y.to_bytes_be()),
        }
    }
}

pub struct Polynomial {
    prime: BigUint,
}

impl Polynomial {
    pub(crate) fn new() -> Self {
        Polynomial {
            prime: PRIME.clone(),
        }
    }

    fn mod_inverse(&self, a: &BigUint, m: &BigUint) -> BigUint {
        a.modpow(&(m - 2u32), m) // Используем малую теорему Ферма для простых модулей
    }

    fn random_polynomial(&self, degree: usize, secret: &BigUint) -> Vec<BigUint> {
        let mut coefficients = vec![secret.clone()];
        for _ in 0..degree {
            let index = BigUint::from_bytes_be(generate_random().as_slice());
            coefficients.push(index);
        }
        coefficients
    }

    fn evaluate_polynomial(&self, coefficients: &[BigUint], x: &BigUint) -> BigUint {
        let mut result = BigUint::zero();
        let mut power = BigUint::one();
        for coeff in coefficients {
            result = (&result + (coeff * &power) % &self.prime) % &self.prime;
            power = (&power * x) % &self.prime;
        }
        result
    }

    pub fn generate_shares(
        &self,
        secret: &BigUint,
        num_shares: usize,
        threshold: usize,
    ) -> Vec<Share> {
        let coefficients = self.random_polynomial(threshold - 1, secret);
        let mut shares = vec![];
        for _x in 1..=num_shares {
            let x = BigUint::from_bytes_be(generate_random().as_slice());
            let y = self.evaluate_polynomial(&coefficients, &x);
            shares.push(Share { x, y });
        }
        shares
    }

    pub fn reconstruct_secret(&self, shares: &Vec<Share>) -> BigUint {
        let mut secret = BigUint::zero();
        for share_i in shares {
            let mut numerator = BigUint::one();
            let mut denominator = BigUint::one();
            for share_j in shares {
                if share_i.x != share_j.x {
                    numerator = (&numerator * &share_j.x) % &self.prime;
                    let diff = if share_j.x > share_i.x {
                        &share_j.x - &share_i.x
                    } else {
                        &self.prime - (&share_i.x - &share_j.x)
                    };
                    denominator = (&denominator * &diff) % &self.prime;
                }
            }
            let lagrange = (&share_i.y * &numerator * self.mod_inverse(&denominator, &self.prime))
                % &self.prime;
            secret = (&secret + &lagrange) % &self.prime;
        }
        secret
    }

    pub fn add_share(&self, shares: &Vec<Share>) -> Share {
        let new_index = BigUint::from_bytes_be(generate_random().as_slice());
        let mut result = BigUint::zero();

        for share_i in shares {
            let mut lambda = BigUint::one();
            for share_j in shares {
                if share_i.x != share_j.x {
                    let numerator = if new_index.clone() >= share_j.x {
                        (new_index.clone() - &share_j.x) % &self.prime
                    } else {
                        (&self.prime - (&share_j.x - new_index.clone()) % &self.prime) % &self.prime
                    };

                    let denominator = if share_i.x >= share_j.x {
                        (&share_i.x - &share_j.x) % &self.prime
                    } else {
                        (&self.prime - (&share_j.x - &share_i.x) % &self.prime) % &self.prime
                    };

                    lambda = (&lambda * &numerator * self.mod_inverse(&denominator, &self.prime))
                        % &self.prime;
                }
            }
            result = (&result + &share_i.y * &lambda) % &self.prime;
        }

        Share {
            x: new_index,
            y: result,
        }
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use num_traits::Num;

    use crate::services::polynomial::Polynomial;

    #[test]
    fn test_polynomial() {
        let sss = Polynomial::new();

        let secret = BigUint::from_str_radix(
            "9c22ff5f21f0b81b113e63f7db6da94fedef11b2119b4088b89664fb9a3cb658",
            16,
        )
        .expect("Invalid secret");
        let num_shares = 5;
        let threshold = 3;

        println!("Original secret: {}", secret);

        let shares = sss.generate_shares(&secret, num_shares, threshold);
        println!("Generated shares: {:?}", shares);

        let mut subset_shares = vec![];

        for i in 0..threshold {
            subset_shares.push(shares[i].clone());
        }

        let reconstructed_secret = sss.reconstruct_secret(&subset_shares.to_vec());
        println!("Reconstructed secret: {}", reconstructed_secret);

        assert_eq!(secret, reconstructed_secret);

        let new_share = sss.add_share(&subset_shares.to_vec());

        println!("New share: {:?}", new_share);

        let subset_shares = vec![new_share, shares[1].clone(), shares[3].clone()];

        let reconstructed_secret = sss.reconstruct_secret(&subset_shares);
        println!("Reconstructed secret: {}", reconstructed_secret);

        assert_eq!(secret, reconstructed_secret);
    }
}
