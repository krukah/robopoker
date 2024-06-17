// root() is just chance node that will deal cards according to some Range<Probability>
enum Range {
    //
    // what is the size/type of range?
    // N = 1326 unique hole cards?
    // N = 169 strategically identical hole cards?
    // N = 52 high cards?
    //
    // what is the domain of the range?
    // sparse, only store nonzeros, hash map?
    // dense, store all N range elements, list?
    //
    // what numeric field are we over, u8 or f32?
    //
    ListWeight([Probability; 1326]),
    ListCombos([u8; 1326]),
    HashWeight(HashMap<[Card; 2], Probability>),
    HashCombos(HashMap<[Card; 2], u8>),
}
