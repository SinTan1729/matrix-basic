//! This is a crate for very basic matrix operations
//! with any type that implement [`Add`], [`Sub`], [`Mul`],
//! [`Zero`], [`Neg`] and [`Copy`]. Additional properties might be
//! needed for certain operations.
//!
//! I created it mostly to learn using generic types
//! and traits.
//!
//! Sayantan Santra (2023)

use errors::MatrixError;
use num::{
    traits::{One, Zero},
    Integer,
};
use std::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Add, Div, Mul, Neg, Sub},
    result::Result,
};

pub mod errors;
mod tests;

/// Trait a type must satisfy to be element of a matrix. This is
/// mostly to reduce writing trait bounds afterwards.
pub trait ToMatrix:
    Mul<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Zero<Output = Self>
    + Neg<Output = Self>
    + Copy
{
}

/// Blanket implementation for [`ToMatrix`] for any type that satisfies its bounds.
impl<T> ToMatrix for T where
    T: Mul<Output = T>
        + Add<Output = T>
        + Sub<Output = T>
        + Zero<Output = T>
        + Neg<Output = T>
        + Copy
{
}

/// A generic matrix struct (over any type with [`Add`], [`Sub`], [`Mul`],
/// [`Zero`], [`Neg`] and [`Copy`] implemented).
/// Look at [`from`](Self::from()) to see examples.
#[derive(PartialEq, Debug, Clone)]
pub struct Matrix<T: ToMatrix> {
    entries: Vec<Vec<T>>,
}

impl<T: ToMatrix> Matrix<T> {
    /// Creates a matrix from given 2D "array" in a [`Vec<Vec<T>>`] form.
    /// It'll throw an error if all the given rows aren't of the same size.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1, 2, 3], vec![4, 5, 6]]);
    /// ```
    /// will create the following matrix:  
    /// ⌈1, 2, 3⌉  
    /// ⌊4, 5, 6⌋
    pub fn from(entries: Vec<Vec<T>>) -> Result<Matrix<T>, MatrixError> {
        let mut equal_rows = true;
        let row_len = entries[0].len();
        for row in &entries {
            if row_len != row.len() {
                equal_rows = false;
                break;
            }
        }
        if equal_rows {
            Ok(Matrix { entries })
        } else {
            Err(MatrixError::UnequalRows)
        }
    }

    /// Returns the height of a matrix.
    pub fn height(&self) -> usize {
        self.entries.len()
    }

    /// Returns the width of a matrix.
    pub fn width(&self) -> usize {
        self.entries[0].len()
    }

    /// Returns the transpose of a matrix.
    pub fn transpose(&self) -> Self {
        let mut out = Vec::new();
        for i in 0..self.width() {
            let mut column = Vec::new();
            for row in &self.entries {
                column.push(row[i]);
            }
            out.push(column)
        }
        Matrix { entries: out }
    }

    /// Returns a reference to the rows of a matrix as `&Vec<Vec<T>>`.
    pub fn rows(&self) -> &Vec<Vec<T>> {
        &self.entries
    }

    /// Return the columns of a matrix as `Vec<Vec<T>>`.
    pub fn columns(&self) -> Vec<Vec<T>> {
        self.transpose().entries
    }

    /// Return true if a matrix is square and false otherwise.
    pub fn is_square(&self) -> bool {
        self.height() == self.width()
    }

    /// Returns a matrix after removing the provided row and column from it.
    /// Note: Row and column numbers are 0-indexed.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();
    /// let n = Matrix::from(vec![vec![5, 6]]).unwrap();
    /// assert_eq!(m.submatrix(0, 0), n);
    /// ```
    pub fn submatrix(&self, row: usize, col: usize) -> Self {
        let mut out = Vec::new();
        for (m, row_iter) in self.entries.iter().enumerate() {
            if m == row {
                continue;
            }
            let mut new_row = Vec::new();
            for (n, entry) in row_iter.iter().enumerate() {
                if n != col {
                    new_row.push(*entry);
                }
            }
            out.push(new_row);
        }
        Matrix { entries: out }
    }

    /// Returns the determinant of a square matrix.
    /// This uses basic recursive algorithm using cofactor-minor.
    /// See [`det_in_field`](Self::det_in_field()) for faster determinant calculation in fields.
    /// It'll throw an error if the provided matrix isn't square.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert_eq!(m.det(), Ok(-2));
    /// ```
    pub fn det(&self) -> Result<T, MatrixError> {
        if self.is_square() {
            // It's a recursive algorithm using minors.
            // TODO: Implement a faster algorithm.
            let out = if self.width() == 1 {
                self.entries[0][0]
            } else {
                // Add the minors multiplied by cofactors.
                let n = 0..self.width();
                let mut out = T::zero();
                for i in n {
                    if i.is_even() {
                        out = out + (self.entries[0][i] * self.submatrix(0, i).det().unwrap());
                    } else {
                        out = out - (self.entries[0][i] * self.submatrix(0, i).det().unwrap());
                    }
                }
                out
            };
            Ok(out)
        } else {
            Err(MatrixError::NotSquare)
        }
    }

    /// Returns the determinant of a square matrix over a field i.e. needs [`One`] and [`Div`] traits.
    /// See [`det`](Self::det()) for determinants in rings.
    /// This method uses row reduction as is much faster.
    /// It'll throw an error if the provided matrix isn't square.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
    /// assert_eq!(m.det_in_field(), Ok(-2.0));
    /// ```
    pub fn det_in_field(&self) -> Result<T, MatrixError>
    where
        T: One,
        T: PartialEq,
        T: Div<Output = T>,
    {
        if self.is_square() {
            // Cloning is necessary as we'll be doing row operations on it.
            let mut rows = self.entries.clone();
            let mut multiplier = T::one();
            let h = self.height();
            let w = self.width();
            for i in 0..(h - 1) {
                // First check if the row has diagonal element 0, if yes, then swap.
                if rows[i][i] == T::zero() {
                    let mut zero_column = true;
                    for j in (i + 1)..h {
                        if rows[j][i] != T::zero() {
                            rows.swap(i, j);
                            multiplier = -multiplier;
                            zero_column = false;
                            break;
                        }
                    }
                    if zero_column {
                        return Ok(T::zero());
                    }
                }
                for j in (i + 1)..h {
                    let ratio = rows[j][i] / rows[i][i];
                    for k in i..w {
                        rows[j][k] = rows[j][k] - rows[i][k] * ratio;
                    }
                }
            }
            for (i, row) in rows.iter().enumerate() {
                multiplier = multiplier * row[i];
            }
            Ok(multiplier)
        } else {
            Err(MatrixError::NotSquare)
        }
    }

    /// Returns the row echelon form of a matrix over a field i.e. needs the [`Div`] trait.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1.0, 2.0, 3.0], vec![3.0, 4.0, 5.0]]).unwrap();
    /// let n = Matrix::from(vec![vec![1.0, 2.0, 3.0], vec![0.0, -2.0, -4.0]]).unwrap();
    /// assert_eq!(m.row_echelon(), n);
    /// ```
    pub fn row_echelon(&self) -> Self
    where
        T: PartialEq,
        T: Div<Output = T>,
    {
        // Cloning is necessary as we'll be doing row operations on it.
        let mut rows = self.entries.clone();
        let mut offset = 0;
        let h = self.height();
        let w = self.width();
        for i in 0..(h - 1) {
            // Check if all the rows below are 0
            if i + offset >= self.width() {
                break;
            }
            // First check if the row has diagonal element 0, if yes, then swap.
            if rows[i][i + offset] == T::zero() {
                let mut zero_column = true;
                for j in (i + 1)..h {
                    if rows[j][i + offset] != T::zero() {
                        rows.swap(i, j);
                        zero_column = false;
                        break;
                    }
                }
                if zero_column {
                    offset += 1;
                }
            }
            for j in (i + 1)..h {
                let ratio = rows[j][i + offset] / rows[i][i + offset];
                for k in (i + offset)..w {
                    rows[j][k] = rows[j][k] - rows[i][k] * ratio;
                }
            }
        }
        Matrix { entries: rows }
    }

    /// Returns the column echelon form of a matrix over a field i.e. needs the [`Div`] trait.
    /// It's just the transpose of the row echelon form of the transpose.
    /// See [`row_echelon`](Self::row_echelon()) and [`transpose`](Self::transpose()).
    pub fn column_echelon(&self) -> Self
    where
        T: PartialEq,
        T: Div<Output = T>,
    {
        self.transpose().row_echelon().transpose()
    }

    /// Returns the reduced row echelon form of a matrix over a field i.e. needs the `Div`] trait.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1.0, 2.0, 3.0], vec![3.0, 4.0, 5.0]]).unwrap();
    /// let n = Matrix::from(vec![vec![1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0]]).unwrap();
    /// assert_eq!(m.reduced_row_echelon(), n);
    /// ```
    pub fn reduced_row_echelon(&self) -> Self
    where
        T: PartialEq,
        T: Div<Output = T>,
    {
        let mut echelon = self.row_echelon();
        let mut offset = 0;
        for row in &mut echelon.entries {
            while row[offset] == T::zero() {
                offset += 1;
            }
            let divisor = row[offset];
            for entry in row.iter_mut().skip(offset) {
                *entry = *entry / divisor;
            }
            offset += 1;
        }
        echelon
    }

    /// Creates a zero matrix of a given size.
    pub fn zero(height: usize, width: usize) -> Self {
        let mut out = Vec::new();
        for _ in 0..height {
            let mut new_row = Vec::new();
            for _ in 0..width {
                new_row.push(T::zero());
            }
            out.push(new_row);
        }
        Matrix { entries: out }
    }

    /// Creates an identity matrix of a given size.
    /// It needs the [`One`] trait.
    pub fn identity(size: usize) -> Self
    where
        T: One,
    {
        let mut out = Matrix::zero(size, size);
        for (i, row) in out.entries.iter_mut().enumerate() {
            row[i] = T::one();
        }
        out
    }

    /// Returns the trace of a square matrix.
    /// It'll throw an error if the provided matrix isn't square.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1, 2], vec![3, 4]]).unwrap();
    /// assert_eq!(m.trace(), Ok(5));
    /// ```
    pub fn trace(self) -> Result<T, MatrixError> {
        if self.is_square() {
            let mut out = self.entries[0][0];
            for i in 1..self.height() {
                out = out + self.entries[i][i];
            }
            Ok(out)
        } else {
            Err(MatrixError::NotSquare)
        }
    }

    /// Returns a diagonal matrix with a given diagonal.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::diagonal_matrix(vec![1, 2, 3]);
    /// let n = Matrix::from(vec![vec![1, 0, 0], vec![0, 2, 0], vec![0, 0, 3]]).unwrap();
    ///
    /// assert_eq!(m, n);
    /// ```
    pub fn diagonal_matrix(diag: Vec<T>) -> Self {
        let size = diag.len();
        let mut out = Matrix::zero(size, size);
        for (i, row) in out.entries.iter_mut().enumerate() {
            row[i] = diag[i];
        }
        out
    }

    /// Multiplies all entries of a matrix by a scalar.
    /// Note that it modifies the supplied matrix.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let mut m = Matrix::from(vec![vec![1, 2, 0], vec![0, 2, 5], vec![0, 0, 3]]).unwrap();
    /// let n = Matrix::from(vec![vec![2, 4, 0], vec![0, 4, 10], vec![0, 0, 6]]).unwrap();
    /// m.mul_scalar(2);
    ///
    /// assert_eq!(m, n);
    /// ```
    pub fn mul_scalar(&mut self, scalar: T) {
        for row in &mut self.entries {
            for entry in row {
                *entry = *entry * scalar;
            }
        }
    }

    /// Returns the inverse of a square matrix. Throws an error if the matrix isn't square.
    /// /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// let m = Matrix::from(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
    /// let n = Matrix::from(vec![vec![-2.0, 1.0], vec![1.5, -0.5]]).unwrap();
    /// assert_eq!(m.inverse(), Ok(n));
    /// ```
    pub fn inverse(&self) -> Result<Self, MatrixError>
    where
        T: Div<Output = T>,
        T: One,
        T: PartialEq,
    {
        if self.is_square() {
            // We'll use the basic technique of using an augmented matrix (in essence)
            // Cloning is necessary as we'll be doing row operations on it.
            let mut rows = self.entries.clone();
            let h = self.height();
            let w = self.width();
            let mut out = Self::identity(h).entries;

            // First we get row echelon form
            for i in 0..(h - 1) {
                // First check if the row has diagonal element 0, if yes, then swap.
                if rows[i][i] == T::zero() {
                    let mut zero_column = true;
                    for j in (i + 1)..h {
                        if rows[j][i] != T::zero() {
                            rows.swap(i, j);
                            out.swap(i, j);
                            zero_column = false;
                            break;
                        }
                    }
                    if zero_column {
                        return Err(MatrixError::Singular);
                    }
                }
                for j in (i + 1)..h {
                    let ratio = rows[j][i] / rows[i][i];
                    for k in i..w {
                        rows[j][k] = rows[j][k] - rows[i][k] * ratio;
                    }
                    // We cannot skip entries here as they might not be 0
                    for k in 0..w {
                        out[j][k] = out[j][k] - out[i][k] * ratio;
                    }
                }
            }

            // Then we reduce the rows
            for i in 0..h {
                if rows[i][i] == T::zero() {
                    return Err(MatrixError::Singular);
                }
                let divisor = rows[i][i];
                for entry in rows[i].iter_mut().skip(i) {
                    *entry = *entry / divisor;
                }
                for entry in out[i].iter_mut() {
                    *entry = *entry / divisor;
                }
            }

            // Finally, we do upside down row reduction
            for i in (1..h).rev() {
                for j in (0..i).rev() {
                    let ratio = rows[j][i];
                    for k in 0..w {
                        out[j][k] = out[j][k] - out[i][k] * ratio;
                    }
                }
            }

            Ok(Matrix { entries: out })
        } else {
            Err(MatrixError::NotSquare)
        }
    }

    // TODO: Canonical forms, eigenvalues, eigenvectors etc.
}

impl<T: Debug + ToMatrix> Display for Matrix<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.entries)
    }
}

impl<T: Mul<Output = T> + ToMatrix> Mul for Matrix<T> {
    // TODO: Implement a faster algorithm.
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        let width = self.width();
        if width != other.height() {
            panic!("row length of first matrix != column length of second matrix");
        } else {
            let mut out = Vec::new();
            for row in self.rows() {
                let mut new_row = Vec::new();
                for col in other.columns() {
                    let mut prod = row[0] * col[0];
                    for i in 1..width {
                        prod = prod + (row[i] * col[i]);
                    }
                    new_row.push(prod)
                }
                out.push(new_row);
            }
            Matrix { entries: out }
        }
    }
}

impl<T: Mul<Output = T> + ToMatrix> Add for Matrix<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        if self.height() == other.height() && self.width() == other.width() {
            let mut out = self.entries.clone();
            for (i, row) in self.rows().iter().enumerate() {
                for (j, entry) in other.rows()[i].iter().enumerate() {
                    out[i][j] = row[j] + *entry;
                }
            }
            Matrix { entries: out }
        } else {
            panic!("provided matrices have different dimensions");
        }
    }
}

impl<T: ToMatrix> Neg for Matrix<T> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let mut out = self;
        for row in &mut out.entries {
            for entry in row {
                *entry = -*entry;
            }
        }
        out
    }
}

impl<T: ToMatrix> Sub for Matrix<T> {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        if self.height() == other.height() && self.width() == other.width() {
            self + -other
        } else {
            panic!("provided matrices have different dimensions");
        }
    }
}

/// Trait for conversion between matrices of different types.
/// It only has a [`matrix_from()`](Self::matrix_from()) method.
/// This is needed since negative trait bound are not supported in stable Rust
/// yet, so we'll have a conflict trying to implement [`From`].
/// I plan to change this to the default From trait as soon as some sort
/// of specialization system is implemented.
/// You can track this issue [here](https://github.com/rust-lang/rust/issues/42721).
pub trait MatrixFrom<T: ToMatrix> {
    /// Method for getting a matrix of a new type from a matrix of type [`Matrix<T>`].
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// use matrix_basic::MatrixFrom;
    ///
    /// let a = Matrix::from(vec![vec![1, 2, 3], vec![0, 1, 2]]).unwrap();
    /// let b = Matrix::from(vec![vec![1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0]]).unwrap();
    /// let c = Matrix::<f64>::matrix_from(a); // Type annotation is needed here
    ///
    /// assert_eq!(c, b);
    /// ```
    fn matrix_from(input: Matrix<T>) -> Self;
}

/// Blanket implementation of [`MatrixFrom<T>`] for converting [`Matrix<S>`] to [`Matrix<T>`] whenever
/// `S` implements [`From(T)`]. Look at [`matrix_into`](Self::matrix_into()).
impl<T: ToMatrix, S: ToMatrix + From<T>> MatrixFrom<T> for Matrix<S> {
    fn matrix_from(input: Matrix<T>) -> Self {
        let mut out = Vec::new();
        for row in input.entries {
            let mut new_row: Vec<S> = Vec::new();
            for entry in row {
                new_row.push(entry.into());
            }
            out.push(new_row)
        }
        Matrix { entries: out }
    }
}

/// Sister trait of [`MatrixFrom`]. Basically does the same thing, just with a
/// different syntax.
pub trait MatrixInto<T> {
    /// Method for converting a matrix [`Matrix<T>`] to another type.
    /// # Example
    /// ```
    /// use matrix_basic::Matrix;
    /// use matrix_basic::MatrixInto;
    ///
    /// let a = Matrix::from(vec![vec![1, 2, 3], vec![0, 1, 2]]).unwrap();
    /// let b = Matrix::from(vec![vec![1.0, 2.0, 3.0], vec![0.0, 1.0, 2.0]]).unwrap();
    /// let c: Matrix<f64> = a.matrix_into(); // Type annotation is needed here
    ///
    ///
    /// assert_eq!(c, b);
    /// ```
    fn matrix_into(self) -> T;
}

/// Blanket implementation of [`MatrixInto<T>`] for [`Matrix<S>`] whenever `T`
/// (which is actually some)[`Matrix<U>`] implements [`MatrixFrom<S>`].
impl<T: MatrixFrom<S>, S: ToMatrix> MatrixInto<T> for Matrix<S> {
    fn matrix_into(self) -> T {
        T::matrix_from(self)
    }
}
