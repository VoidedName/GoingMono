mod geometry2d;
mod r_tree;

use crate::geometry2d::{Point, Rectangle};
use crate::r_tree::{ObjectRecord, RTree};

fn main() {
    let records = vec![
        ObjectRecord(
            Rectangle {
                low: Point::new(0.0, 0.0),
                high: Point::new(10.0, 10.0),
            },
            1,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(12.0, 0.0),
                high: Point::new(15.0, 15.0),
            },
            2,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 7.0),
                high: Point::new(14.0, 14.0),
            },
            3,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(10.0, 11.0),
                high: Point::new(11.0, 12.0),
            },
            4,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(4.0, 4.0),
                high: Point::new(5.0, 6.0),
            },
            5,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(4.0, 9.0),
                high: Point::new(5.0, 11.0),
            },
            6,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(13.0, 0.0),
                high: Point::new(14.0, 1.0),
            },
            7,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(13.0, 13.0),
                high: Point::new(16.0, 16.0),
            },
            8,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(2.0, 13.0),
                high: Point::new(4.0, 16.0),
            },
            9,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(2.0, 2.0),
                high: Point::new(3.0, 3.0),
            },
            10,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(10.0, 0.0),
                high: Point::new(12.0, 5.0),
            },
            11,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 3.0),
                high: Point::new(8.0, 6.0),
            },
            12,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 3.0),
                high: Point::new(8.0, 6.0),
            },
            13,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 3.0),
                high: Point::new(8.0, 6.0),
            },
            14,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 3.0),
                high: Point::new(8.0, 6.0),
            },
            15,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 3.0),
                high: Point::new(8.0, 6.0),
            },
            16,
        ),
        ObjectRecord(
            Rectangle {
                low: Point::new(7.0, 3.0),
                high: Point::new(8.0, 6.0),
            },
            17,
        ),
    ];

    let mut tree = RTree::new(8, 2).unwrap();

    for rec in records.iter() {
        tree.insert(rec.clone());
    }

    println!("{}", tree);
    println!(
        "{:?}",
        tree.search_area(&Rectangle {
            low: Point::new(5.0, 5.0),
            high: Point::new(7.0, 7.0)
        })
    );
    println!("{:?}", tree.search_point(&Point::new(5.0, 5.0)));

    tree.delete(records[0].clone());
    tree.delete(records[1].clone());
    tree.delete(records[2].clone());
    tree.delete(records[3].clone());
    tree.delete(records[4].clone());
    tree.delete(records[5].clone());
    tree.delete(records[6].clone());
    tree.delete(records[7].clone());
    tree.delete(records[8].clone());
    tree.delete(records[9].clone());
    tree.delete(records[10].clone());
    tree.delete(records[11].clone());
    tree.delete(records[12].clone());
    tree.delete(records[13].clone());
    tree.delete(records[14].clone());
    tree.delete(records[15].clone());
    tree.delete(records[16].clone());

    println!("{}", tree);
}
