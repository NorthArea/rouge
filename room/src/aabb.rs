use macroquad::prelude::*;

/// Осевой параллелепипед (AABB) — вся геометрия комнаты для столкновений (D-43, D-44): не `trait
/// Collider`, не список тел с решателем, а тип из двух точек и функция из трёх сравнений.
#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Пересечение по всем трём осям одновременно. Сравнение строгое: коробки, соприкасающиеся
    /// гранью, не считаются пересекающимися — удобнее для разрешения столкновений по осям
    /// (`T-ROOM-8`), где движение должно быть в состоянии довести игрока вплотную к стене.
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_nested_boxes_intersect() {
        let outer = Aabb::from_center_half_extents(vec3(0.0, 0.0, 0.0), vec3(2.0, 2.0, 2.0));
        let inner = Aabb::from_center_half_extents(vec3(0.0, 0.0, 0.0), vec3(0.5, 0.5, 0.5));
        assert!(outer.intersects(&inner), "вложенные коробки не пересеклись");
    }

    #[test]
    fn two_far_apart_boxes_do_not_intersect() {
        let a = Aabb::from_center_half_extents(vec3(-10.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        let b = Aabb::from_center_half_extents(vec3(10.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        assert!(
            !a.intersects(&b),
            "разнесённые коробки посчитаны пересекающимися"
        );
    }

    #[test]
    fn boxes_overlapping_on_x_and_y_but_separated_on_z_do_not_intersect() {
        let a = Aabb::from_center_half_extents(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        let b = Aabb::from_center_half_extents(vec3(0.0, 0.0, 10.0), vec3(1.0, 1.0, 1.0));
        assert!(
            !a.intersects(&b),
            "коробки, разнесённые по Z, посчитаны пересекающимися"
        );
    }

    #[test]
    fn boxes_overlapping_on_y_and_z_but_separated_on_x_do_not_intersect() {
        let a = Aabb::from_center_half_extents(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        let b = Aabb::from_center_half_extents(vec3(10.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        assert!(
            !a.intersects(&b),
            "коробки, разнесённые по X, посчитаны пересекающимися"
        );
    }

    #[test]
    fn boxes_overlapping_on_x_and_z_but_separated_on_y_do_not_intersect() {
        let a = Aabb::from_center_half_extents(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        let b = Aabb::from_center_half_extents(vec3(0.0, 10.0, 0.0), vec3(1.0, 1.0, 1.0));
        assert!(
            !a.intersects(&b),
            "коробки, разнесённые по Y, посчитаны пересекающимися"
        );
    }

    #[test]
    fn boxes_touching_at_a_face_do_not_intersect() {
        // Грань первой коробки (x = 1) точно совпадает с гранью второй (x = 1) — решение позиции
        // зафиксировано этим тестом: строгое сравнение, "не пересекаются".
        let a = Aabb::from_center_half_extents(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        let b = Aabb::from_center_half_extents(vec3(2.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        assert!(
            !a.intersects(&b),
            "соприкасающиеся гранью коробки посчитаны пересекающимися"
        );
    }
}
