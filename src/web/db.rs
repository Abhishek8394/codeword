use super::OnlinePlayer;

#[derive(Clone)]
pub struct RedisGameDB {}

// #[derive(Clone)]
// pub struct InMemGameDB<P: OnlinePlayer> {
//     db: Arc<RwLock<HashMap<String, ArcGameWrapper<P>>>>,
// }

// impl<P: OnlinePlayer> InMemGameDB<P> {
//     pub fn new() -> Self {
//         GameDB {
//             db: Arc::new(RwLock::new(HashMap::new())),
//         }
//     }

//     pub fn get_num_games(&self) -> Result<usize> {
//         Ok(self.db.read().unwrap().len())
//     }

//     pub fn add_new_game(&self, game_id: &str, game: GameWrapper<P>) -> Result<()> {
//         eprintln!("Adding game: {}", game_id);
//         match self.db.write() {
//             Ok(mut db) => {
//                 db.insert(String::from(game_id), get_arc_game_wrapper(game));
//             }
//             Err(_e) => {
//                 bail!("cannot add game now");
//             }
//         };
//         Ok(())
//     }

//     pub fn get_game(&self, game_id: &str) -> Result<ArcGameWrapper<P>> {
//         match self.db.read() {
//             Ok(h_map) => match h_map.get(game_id) {
//                 Some(arc_game) => {
//                     return Ok(arc_game.clone());
//                 }
//                 None => {
//                     bail!(format!("No game found for: {:}", game_id));
//                 }
//             },
//             Err(e) => {
//                 eprintln!("{:?}", e);
//                 bail!("Error reading DB")
//             }
//         }
//     }
// }
