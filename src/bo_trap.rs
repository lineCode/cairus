/*
input: list of edges
output: list of trapezoids

Sweep line is a horizontal line going from top (minimum y) to bottom (maximum y)

LineSegment defined in common_geometry.rs contains 2 points
edge is a line + top, bot, dir
    dir is a direction and should come from whatever initially 'drew' the lines
        in a pinch, we could generate dir from a sequence of line segments assuming
        each segment's first point is the previous segment's end point.
        dir should be +1 for a segment that is being drawn in the positive y direction,
        -1 for a segment being drawn in the negative y direction, and 0 for horizontal lines
        (horizontal lines don't actually matter since we will never cross them with a
        horizontal ray)
    For example: A clockwise drawn square would have a right side with a +1 dir,
        and a left side with a -1 dir.
SL_edge has edge + *prev, *next, *colinear, deferred_trap (top, *right)

1. build event queue (EQ) (BST?)
    add event for each endpoint of lines in edge list.
        min(y of points) is START, max is END
    event is a point and associated edge or two and an enum event type
    sort events by point.y first, then by edge (top bottom, left right)

2. initialize sweep line list (SLL)
    SLL starts empty. Contains SL_edges. Is doubly linked list
    SL has *head, y, *current_SL_edge
    ? what about multiple lines intersecting at the same point?

while EQ not empty:
    Pop event off EQ.
    Set SL.y = event.y
    Process event:
        case: event.type = start
            insert event.edge into SLL (build SL_edge)
                building SL_edge:
                    SL_edge->edge = event.edge
                    if SL_edge->next != null start new trap:
                        SL_edge.deferred_trap->right = SL_edge->next.edge
                        SL_edge.deferred_trap.top = SL.y
                    if SL_edge->prev.deferred_trap.right != null (edge to left has
                                                deferred trap)
                        add_to_traps(SL_edge->prev, SL.y)
                    SL_edge->prev.deferred_trap.right = SL_edge
                    SL_edge->prev.deferred_trap.top = SL.y
            check if SL_edge.prev intersects with SL_edge
                add intersection to EQ
            check if SL_edge.next intersects with SL_edge
                add intersection to EQ (future? current?)

        case: event.type = end
            if SL_edge->prev intersects with SL_edge->next
                add intersection to EQ if it isn't already there
            if SL_edge.deferred_trap->right != null
                add_to_traps(SL_edge, SL.y)
            if SL_edge->prev.deferred_trap->right != null (should never be null
                                prob just check SL_edge->prev != null)
                add_to_traps(SL_edge->prev, SL.y)
                SL_edge->prev.deferred_trap->right = SL_edge.deferred_trap->right
                SL_edge->prev.deferred_trap.top = SL.y
            remove SL_edge from SLL:
                SL_edge->prev = SL_edge->next
                SL_edge->next = SL_edge->prev

        case: event.type = intersection
            if SL_edgeL.deferred_trap->right != null (should be SL_edgeR.edge)
                add_to_traps(SL_edgeL, SL.y)
            SL_edgeL.deferred_trap->right = SL_edgeR.deferred_trap->right
            SL_edgeL.deferred_trap.top = SL.y
            if SL_edgeR.deferred_trap->right != null
                add_to_traps(SL_edgeR, SL.y)
            SL_edgeR.deferred_trap->right = SL_edgeL->edge
            SL_edgeR.deferred_trap.top = SL.y
            if SL_edgeL->prev.deferred_trap->right != null (should be SL_edgeL.edge)
                add_to_traps(SL_edgeL->prev, SL.y)
            SL_edgeL->prev.deferred_trap->right = SL_edgeR->edge
            SL_edgeL->prev.deferred_trap.top = SL.y
            swap SL_edgeL and SL_edgeR:
                SL_edgeL->prev->next = SL_edgeR (if L->prev == null, SL->head = R)
                SL_edgeR->prev = SL_edgeL->prev
                SL_edgeL->next = SL_edgeR->next
                SL_edgeL->prev = SL_edgeR
                SL_edgeR->next->prev = SL_edgeL (if R->next != null)
                SL_edgeR->next = SL_edgeL
            check if SL_edgeR.prev intersects with SL_edgeR
                add intersection to EQ
            check if SL_edgeL.next intersects with SL_edgeL
                add intersection to EQ

In case of multiple lines crossing at same intersection point we have a couple problems:
    1. if order of event insertion is wrong, we may end up with non-adjacent edges in SLL being
        swapped
    2. we end up in an infinite loop adding the same intersections to the event queue over and over
does slope of lines help with this? investigate cairo code...

*/
/*
add_to_traps(SL_edge edge, float bot, int mask, traps *traps)
    //mask is 0xFFFFFFFF if using winding rule, 0x1 if using even/odd rule
    //only output traps with positive area
    if edge.deferred_trap.top >= bot
        return
    //count edge directions for ray right to infinity
    in_out = 0
    pos = edge.deferred_trap->right (or pos = edge->next? should be same, no?)
    while (pos != null)
        in_out += pos.dir
        pos = pos.deferred_trap->right (or pos = pos->next? should be same, no?)
    //in_out & mask is zero means do not fill (0 or even)
    if in_out & mask != 0
        LineSegment left, right
        left = edge->LineSegment
        right = edge.deferred_trap->right->LineSegment
        traps_push(left, right, edge.deferred_trap.top, bot)
*/
use common_geometry::{Edge, Point, LineSegment};
use std::cmp::Ordering;
use std::clone::Clone;
use trapezoid_rasterizer::Trapezoid;
extern crate linked_list;
use self::linked_list::{LinkedList, Cursor};

#[derive(Eq, PartialEq, Debug)]
pub enum EventType {
    Start,
    End,
    Intersection
}

impl PartialOrd for EventType {
    fn partial_cmp(&self, other: &EventType) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for EventType {
    fn cmp(&self, other: &EventType) -> Ordering {
        match *self {
            EventType::Start =>
                match *other {
                    EventType::Start => Ordering::Equal,
                    EventType::End => Ordering::Greater,
                    EventType::Intersection => Ordering::Greater,
                },
            EventType::End =>
                match *other {
                    EventType::Start => Ordering::Less,
                    EventType::End => Ordering::Equal,
                    EventType::Intersection => Ordering::Less,
                },
            EventType::Intersection =>
                match *other {
                    EventType::Start => Ordering::Less,
                    EventType::End => Ordering::Greater,
                    EventType::Intersection => Ordering::Equal,
                },
        }
    }
}

pub struct Event {
    edge_left: Edge,
    edge_right: Option<Box<Edge>>,
    point: Point,
    event_type: EventType
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        let y_compare = self.point.y.partial_cmp(&other.point.y).unwrap_or(Ordering::Equal);
        if y_compare != Ordering::Equal   {
                return y_compare
        }

        let x_compare = self.point.x.partial_cmp(&other.point.x).unwrap_or(Ordering::Equal);
        if x_compare != Ordering::Equal   {
                return x_compare
        }

        let type_compare = self.event_type.cmp(&other.event_type);
        if type_compare == Ordering::Equal {
            return Ordering::Greater
        }
        type_compare
    }
}

// Need to check this code
impl PartialEq for Event {
    fn eq(&self, other:&Event) -> bool {
        true
    }
}

impl Eq for Event {}

impl Event {
    fn new(edge_left: Edge, point: &Point, event_type: EventType) -> Event {
        Event {
            point: *point,
            edge_left: edge_left,
            edge_right: None,
            event_type: event_type,
        }
    }
}

fn event_list_from_edges(edges: Vec<Edge>) -> Vec<Event> {
    let mut events = Vec::new();
    for edge in edges {
        if edge.top == edge.bottom {
            // Is horizontal
            if edge.line.point1.x < edge.line.point2.x {
                // let start_event = Event::new();
                events.push(Event::new(edge,
                                       &Point::new(edge.line.point1.x, edge.line.point1.y),
                                       EventType::Start));
                events.push(Event::new(edge,
                                       &Point::new(edge.line.point2.x, edge.line.point2.y),
                                       EventType::End));
            }
            else {
                events.push(Event::new(edge,
                                       &Point::new(edge.line.point2.x, edge.line.point2.y),
                                       EventType::Start ));
                events.push(Event::new(edge,
                                       &Point::new(edge.line.point1.x, edge.line.point1.y),
                                       EventType::End ));
            }
        }

        if edge.top == edge.line.point1.y {
            // Point1 is start event
            events.push(Event::new(edge,
                                   &Point::new(edge.line.point1.x, edge.line.point1.y),
                                   EventType::Start ));
            events.push(Event::new(edge,
                                   &Point::new(edge.line.point2.x, edge.line.point2.y),
                                   EventType::End ));

        } else {
            // Point2 is start event
            events.push(Event::new(edge,
                                   &Point::new(edge.line.point2.x, edge.line.point2.y),
                                   EventType::Start ));
            events.push(Event::new(edge,
                                   &Point::new(edge.line.point1.x, edge.line.point1.y),
                                   EventType::End ));
        }
    }
    events.sort();
    events
}

/// Defines a ScanLineEdge for our ScanLineList
///
/// The ScanLineEdges will be used to create trapezoids.
/// Top will be set by our ScanLine to mark the top of our trapezoid.
/// Left will be set based on the leftmost point of our line to determine where in our ScanLineList
///     we need to insert our ScanLineEdge. This is used for sorting our ScanLineList and is updated
///     when it intersects another line.
/// Line is our current line.
/// Note: We may need to add a Right (right: Option<Box<LineSegment>>) to track the right side of
///     our trapezoid but for now we will let the ScanLineList determine this based on if there is a
///     ScanLineEdge after the current ScanLineEdge in our ScanLineList.
#[derive(Debug, Copy, Clone)]
pub struct ScanLineEdge {
    top: f32,
    left: f32,
    line: LineSegment,
}

impl ScanLineEdge {
    fn new(top: f32, left: f32, line: LineSegment) -> ScanLineEdge {
        ScanLineEdge {
            top: top,
            left: left,
            line: line,
        }
    }

    /// Returns the x value on the line that intersects with the current y value.
    pub fn current_x_for_y(&self, y: f32) -> f32 {
        let min = self.line.min_y_point();
        (y - min.y) / self.line.slope() + min.x
    }
}

#[derive(Debug)]
pub struct ScanLine {
    y: f32,
    index: Option<Box<i32>>,
}

impl ScanLine {
    fn new(y: f32) -> ScanLine{
        ScanLine {
            y: y,
            index: None,
        }
    }
}

/// Scan will loop over all of the Edges in the vector and build Trapezoids out of them.
pub fn scan(edges: Vec<Edge>) -> Vec<Trapezoid> {
    // Create the empty Scan Line Linked List
    let mut sl_list: LinkedList<ScanLineEdge> = LinkedList::new();
    // Create a cursor to move over the list
    let mut cursor = sl_list.cursor();
    // Create the list of events
    let mut events = event_list_from_edges(edges);
    // Keep looping until the Event List is empty
    while !events.is_empty() {
        // Get the current event
        let event = events.remove(0);
        // Set the scan line to the events y value
        let scan_line = event.point.y;

        // Process Event
        // START CASE
        if event.event_type == EventType::Start{
//            println!("adding SLEdge");
//            // find the left most point of the edge_left line
//            let left = event.edge_left.line.min_x_point().x;
//            // create a new node and add it to the list
//            let mut sl_edge = ScanLineEdge::new(scan_line, left, event.edge_left.line);
//
//            // Insert the node into the linked list. Need to work on the logic for where to add it.
//            if cursor.peek_next().is_none() {
//                // if the next is empty we check our previous to see if its also empty
//                // if it is we insert, otherwise we move our cursor back on position
//                if cursor.peek_prev().is_none() {
//                    cursor.insert(sl_edge);
//                }
//            // if the list is not empty we need to find where to put the element
//            } else {
//                if cursor.peek_next().is_none() {
//                    cursor.prev();
//                }
//                let mut insert = false;
//                while !insert {
//                    if cursor.peek_next().is_none() {
//                        insert == true;
//                    } else {
//                        let result = find_line_place(event.point, event.edge_left.line, *cursor.peek_next().unwrap());
//                        if result == Comparator::Greater {
//                            cursor.next();
//                        } else if result == Comparator::Less {
//                            // if its less then the next we need to see if it is also greater then the previous
//                            if cursor.peek_prev().is_none() {
//                                insert = true;
//                            } else {
//                                let result2 = find_line_place(event.point, event.edge_left.line, *cursor.peek_prev().unwrap());
//                                if result2 == Comparator::Greater {
//                                    insert == true;
//                                } else {
//                                    cursor.prev();
//                                }
//                            }
//                        } else if result == Comparator::Equal {
//                            // this case means the line is already in our list so we dont add it
//                            break;
//                        } else if result == Comparator::Empty {}
//                    }
//
//                }
//                cursor.insert(sl_edge);
//            }
            // find the left most point of the edge_left line
            let left = event.edge_left.line.min_x_point().x;
            // create a new node and add it to the list
            let mut sl_edge = ScanLineEdge::new(scan_line, left, event.edge_left.line);
            // Set the cursor back to the beginning
            cursor.reset();
            if cursor.peek_next().is_none() {
                cursor.insert(sl_edge);
            } else {
                while find_line_place(event.point, event.edge_left.line, *cursor.peek_next().unwrap()) == Comparator::Greater {
                    cursor.next();
                    if cursor.peek_next().is_none() {
                        break;
                    }
                }
                cursor.insert(sl_edge);
            }


            println!("Added Start to the scan line at y: {}", scan_line);
            println!("current x, y value: {} {}",cursor.next().unwrap().current_x_for_y(scan_line), scan_line );
        }

        // END CASE
        else if event.event_type == EventType::End {
        // how do we know which event to remove?
            // when we call remove on the cursor it will remove the next element.
            // when we call cursor.next or cursor.prev it moves the cursor left or right
            // when we call cursor.peek_left or right it gets the next element without moving the cursor
            // the events will always be sorted by the current left point
            // We know what line to remove based on the current event which will tell us what that
            // left point will be

            // REMOVE FROM SL_LIST
            // if our event line is equal to our cursor_left line then see if our lines are equal, if yes remove
            // if no then we need to see which direction to move...
            // if our event line is greater then our cursor left line then we need to move right and repeat
            // if our event line is less then our cursor left line then we need to move left
            let mut result = Comparator::Empty;
            // ***** need to remove after i fix a bug *****
            cursor.reset();
            while result != Comparator::Equal {
                // Not sure if i need this. could if the cursor is at the end of the list
                if cursor.peek_next().is_none() {
                    cursor.prev();
                }
                result = find_line_place(event.point, event.edge_left.line, *cursor.peek_next().unwrap());
                // Code just for testing and debugging
                match result{
                    Comparator::Greater => println!("Next is Greater"),
                    Comparator::Less => println!("Next is Less"),
                    Comparator::Equal => println!("Next is Equal"),
                    Comparator::Empty => println!("Next is Empty"),
                }
                if result == Comparator::Equal {
                    break;
                } else if result == Comparator::Greater {
                    cursor.prev();
                } else if result == Comparator::Greater {
                    cursor.next();
                } else {
                    println!("Failed to remove a SL_Edge from the List");
                    break;
                }

            }
            cursor.remove();
            // before we remove we need to build possible trapezoids for both the left and right
            // could get complicated since we cant move the cursor easily.
        }

        // print the Scan Line List
        cursor.reset();
        let mut index = 0;
        while cursor.peek_next().is_some(){
            println!("Index {}:  y:{}", index, cursor.peek_next().unwrap().top);
            index = index + 1;
            cursor.next();
        }


        println!("Scan Line: {}", scan_line);
    }
//    println!("SLL: {:?}", sl_list);

   Vec::new()
}

#[derive(Eq, PartialEq, Debug)]
pub enum Comparator {
    Greater,
    Less,
    Equal,
    Empty,
}

// need to rename function. it will compare a line to the next one in the list
// may want to pass in a point as well so that we can use this same function for insert
pub fn find_line_place(point: Point, line: LineSegment, next_sl_edge : ScanLineEdge) -> Comparator {
    let next_line = next_sl_edge.line;

    // if the lines are the same line we return equal because we have a duplicate
    if line == next_line {
        return Comparator::Equal;
    }
    // Get the point on the next line for the current y value we are at since that is how the
    // linked list is sorted.
    let next_x = next_sl_edge.current_x_for_y(point.y);
    // if the point is the same as the next point or lines intersect and we need to look at the
    // slope to determine the sorting order. We already know they have the same y value so we just
    // look at the x values
    if point.x == next_x {
        // compare the slopes of the lines
        if line.slope() > next_line.slope() {
            return Comparator::Greater;
        }
        else {
            return Comparator::Less;
        }
        // if the point is not on the nextLine we just need to see if it comes before or after
    } else if point.x > next_x {
        return Comparator::Greater;
    } else {
        return Comparator::Less;
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use common_geometry::{LineSegment, Point, Edge};
    use std::cmp::Ordering;

    fn create_edge(x1: f32, y1: f32, x2: f32, y2:f32) -> Edge{
        let mut top = y1;
        let mut bottom = y2;
        if y1 > y2 {
            top = y2;
            bottom = y1;
        }

        Edge{
            line: LineSegment::new(x1, y1, x2, y2),
            top: top,
            bottom: bottom,
            direction: 1,

        }
    }

    fn create_start_event(x1: f32, y1: f32, x2:f32, y2:f32) -> Event {
        let edge = create_edge(x1, y1, x2, y2);
        let point = Point::new(x1, y1);
        Event::new(edge, &point, EventType::Start)
    }

    #[test]
    fn event_compare_y_lesser(){
        let lesser = create_start_event(0., 0., 3., 3.);
        let greater = create_start_event(1., 1., 0., 2.);
        assert_eq!(lesser.cmp(&greater), Ordering::Less);
    }

    #[test]
    fn event_compare_y_greater(){
        let lesser = create_start_event(0., 0., 3., 3.);
        let greater = create_start_event(1., 1., 0., 2.);
        assert_eq!(greater.cmp(&lesser), Ordering::Greater);
    }

    #[test]
    fn event_compare_x_lesser(){
        let lesser = create_start_event(0., 0., 0., 0.);
        let greater = create_start_event(1., 0., 0., 0.);
        assert_eq!(lesser.cmp(&greater), Ordering::Less);
    }

    #[test]
    fn event_compare_x_greater(){
        let lesser = create_start_event(0., 0., 0., 0.);
        let greater = create_start_event(1., 0., 0., 0.);
        assert_eq!(greater.cmp(&lesser), Ordering::Greater);
    }

    #[test]
    fn event_compare_type_greater(){
        let dummy = create_start_event(0., 0., 0., 0.);
        assert_eq!(dummy.cmp(&dummy), Ordering::Greater);
    }

    #[test]
    fn event_sorting() {
        let mut event_list = vec![
            create_start_event(0., 1., 0., 3.),
            create_start_event(0., 0., 1., 2.),
            create_start_event(0., 0., 0., 1.)
        ];

        event_list.sort();
        assert_eq!(event_list.get(0).unwrap().edge_left.line.point2.y, 1.);
        assert_eq!(event_list.get(1).unwrap().edge_left.line.point2.y, 2.);
        assert_eq!(event_list.get(2).unwrap().edge_left.line.point2.y, 3.);
    }


    #[test]
    fn event_list_from_edges_sorted_test_size() {
        // Verify event list is the correct size
        let edges = vec![
            create_edge(3., 4., 1., 2.),
            create_edge(0., 1., 6., 6.),
            create_edge(0., 0., 5., 5.),
        ];

        let event_list = event_list_from_edges(edges);
        assert_eq!(event_list.len(), 6);
    }

    #[test]
    fn event_list_from_edges_sorted_test_order() {
        // Verify event list is the correct order
        let edges = vec![
        create_edge(3., 4., 1., 2.),
        create_edge(0., 1., 6., 6.),
        create_edge(0., 0., 5., 5.),
        ];

        let event_list = event_list_from_edges(edges);
        assert_eq!(event_list.get(0).unwrap().point, Point::new(0., 0.));
        assert_eq!(event_list.get(1).unwrap().point, Point::new(0., 1.));
        assert_eq!(event_list.get(2).unwrap().point, Point::new(1., 2.));
        assert_eq!(event_list.get(3).unwrap().point, Point::new(3., 4.));
        assert_eq!(event_list.get(4).unwrap().point, Point::new(5., 5.));
        assert_eq!(event_list.get(5).unwrap().point, Point::new(6., 6.));
    }

    #[test]
    fn event_list_from_edges_sorted_test_types() {
        // Verify event list events have the correct start/end types
        let edges = vec![
        create_edge(3., 4., 1., 2.),
        create_edge(0., 1., 6., 6.),
        create_edge(0., 0., 5., 5.),
        ];

        let event_list = event_list_from_edges(edges);
        assert_eq!(event_list.get(0).unwrap().event_type, EventType::Start);
        assert_eq!(event_list.get(1).unwrap().event_type, EventType::Start);
        assert_eq!(event_list.get(2).unwrap().event_type, EventType::Start);
        assert_eq!(event_list.get(3).unwrap().event_type, EventType::End);
        assert_eq!(event_list.get(4).unwrap().event_type, EventType::End);
        assert_eq!(event_list.get(5).unwrap().event_type, EventType::End);
    }


    #[test]
    fn event_constructor() {
        let edge = create_edge(0., 0., 0., 0.);
        let point = Point{x: 0., y: 0.};
        let event = Event::new(edge, &point, EventType::Start);
        assert_eq!(event.edge_left.line.point1, edge.line.point1);
        assert_eq!(event.point, point);
        assert_eq!(event.event_type, EventType::Start);
    }

    #[test]
    fn scan_test() {
        let edges = vec![
        create_edge(0., 0., 5., 5.),
        create_edge(3., 4., 1., 2.),
        create_edge(0., 1., 6., 6.),
        ];

        scan(edges);
    }
}