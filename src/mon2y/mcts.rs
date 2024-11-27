// Iterate

use super::node::Node;

fn iterate(node: &mut Node) {
    if let Some(selection) = node.selection() {
        selection.expansion();
        let reward = selection.play_out();
        selection.back_propogate(reward);
    }
}
