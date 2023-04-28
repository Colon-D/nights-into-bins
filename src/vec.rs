use num_traits::Num;

#[derive(Clone, Copy)]
pub struct Vec2<Type> {
    pub x: Type,
    pub y: Type,
}

impl<T> std::ops::Mul<T> for Vec2<T>
where
    T: Num + Copy,
{
    type Output = Vec2<T>;

    fn mul(self, scalar: T) -> Vec2<T> {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

pub struct Vec3<Type> {
    pub x: Type,
    pub y: Type,
    pub z: Type,
}

impl<T> Vec3<T>
where
    T: Num + Copy,
{
    pub fn cross(&self, other: &Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn dot(&self, other: &Vec3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn to<U: From<T>>(&self) -> Vec3<U> {
        Vec3 {
            x: U::from(self.x),
            y: U::from(self.y),
            z: U::from(self.z),
        }
    }
}

impl<T> std::ops::Add<&Vec3<T>> for &Vec3<T>
where
    T: Num + Copy,
{
    type Output = Vec3<T>;

    fn add(self, other: &Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T> std::ops::Add<Vec3<T>> for Vec3<T>
where
    T: Num + Copy,
{
    type Output = Vec3<T>;

    fn add(self, other: Vec3<T>) -> Vec3<T> {
        &self + &other
    }
}

impl<T> std::ops::Sub<&Vec3<T>> for &Vec3<T>
where
    T: Num + Copy,
{
    fn sub(self, other: &Vec3<T>) -> Vec3<T> {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    type Output = Vec3<T>;
}

impl<T> std::ops::Sub<Vec3<T>> for Vec3<T>
where
    T: Num + Copy,
{
    type Output = Vec3<T>;

    fn sub(self, other: Vec3<T>) -> Vec3<T> {
        &self - &other
    }
}
