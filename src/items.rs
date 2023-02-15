use crate::vector_types::Vec2;

use rlua::Lua;
use std::fs;

pub enum ItemType {
    Air,
    BlockCube,
    BlockCross,
    UserItem,
}

pub struct ItemData {
    pub item_type:     ItemType,
    pub is_transparent: bool,
    pub show_in_inventory: bool,
    pub name: String,
    pub top_tex_coords: Vec2<u16>,
    pub side_tex_coords: Vec2<u16>,
    pub bottom_tex_coords: Vec2<u16>,
}

pub struct ItemManager {
    items: Vec<ItemData>,
}

impl ItemManager {
    /// Creates an empty ItemManager
    pub fn new() -> ItemManager {
        ItemManager { items: Vec::new() }
    }

    /// Inserts a new item into the ItemManager
    pub fn put_new_item(&mut self, item: ItemData) {
        self.items.push(item);
    }

    /// Gets item info from id
    pub fn get_item_by_id(&self, id: i32) -> Option<&ItemData> {
        if id < self.items.len() as i32 {
            return Option::Some(self.items.get(id as usize).unwrap());
        }

        Option::None
    }

    /// Gets item id from name
    pub fn get_id_by_name(&self, name: String) -> Option<i32> {
        for i in 0..self.items.len() {
            if self.items.get(i).unwrap().name == name {
                return Some(i as i32);
            }
        }
        None
    }

    /// Runs the lua script at the `path` and inserts the new items into the `item_manager`
    pub fn load_items(&mut self, path: String) {
        let asset_script = fs::read_to_string(path).expect("Unable to load loadAssetInfo script");

        let lua = Lua::new();

        lua.context(|lua_ctx| {
            let globals = lua_ctx.globals(); // Get globals from lua

            lua_ctx.scope(|scope| {

                let add_asset = // Create a function that takes in all info and compiles it into a ItemData struct
                    scope.create_function_mut(|_, (item_name, item_type_str, is_transparent, show_in_inventory, coords): (String, String, bool, bool, Vec<u16>)| {
                        // println!("New Asset: {}, {}, {}, {}", item_name, item_type_str, is_transparent, show_in_inventory);

                        let item_type: ItemType;
                        match item_type_str.as_str() {
                            "Air" => item_type = ItemType::Air,
                            "BlockCube" => item_type = ItemType::BlockCube,
                            "BlockCross" => item_type = ItemType::BlockCross,
                            "UserItem" => item_type = ItemType::UserItem,
                            _ => item_type = ItemType::UserItem,
                        }

                        let new_item = ItemData {
                            item_type,
                            is_transparent,
                            show_in_inventory,
                            name: item_name,
                            top_tex_coords: Vec2::new(coords[0] as u16, coords[1] as u16),
                            side_tex_coords: Vec2::new(coords[2] as u16, coords[3] as u16),
                            bottom_tex_coords: Vec2::new(coords[4] as u16, coords[5] as u16),
                        };

                        self.put_new_item(new_item);

                        Ok(())
                    }).unwrap();
                globals.set("add_asset", add_asset).unwrap();

                let set_atlas = // Sets which atlas the texture is in
                    lua_ctx.create_function(|_, (atlas_path, width, height): (String, u16, u16)| {
                        println!("Set Atlas: {}, {}, {}", atlas_path, width, height);

                        Ok(())
                    }).unwrap();
                globals.set("set_atlas", set_atlas).unwrap();

                lua_ctx.load(
                    r#"
                        item_name = "UNKNOWN"
                        item_type = "UserItem"
                        is_transparent = false
                        show_in_inventory = true
                        top_coord_x, top_coord_y = 0, 0
                        side_coord_x, side_coord_y = 0, 0
                        bottom_coord_x, bottom_coord_y = 0, 0

                        function setInfo(name, itemType, isTransparent, showInInventory)
                            item_name = name or "UNKNOWN"
                            item_type = itemType or "UserItem"
                            is_transparent = isTransparent or false
                            show_in_inventory = showInInventory or true
                        end

                        function setCoords(topX, topY, sideX, sideY, bottomX, bottomY)
                            top_coord_x = topX or 0
                            top_coord_y = topY or 0
                            side_coord_x = sideX or top_coord_x
                            side_coord_y = sideY or top_coord_y
                            bottom_coord_x = bottomX or top_coord_x
                            bottom_coord_y = bottomY or top_coord_y
                        end

                        function pushItem()
                            add_asset(item_name, item_type, is_transparent, show_in_inventory, {top_coord_x, top_coord_y, side_coord_x, side_coord_y, bottom_coord_x, bottom_coord_y}) -- Change to pull from global variables
                        end

                        function setAtlas(path, width, height)
                            width = width or 10
                            height = height or 10
                            set_atlas(path, width, height)
                        end
                    "#
                )
                .set_name("Load Asset Functions").unwrap()
                .exec()
                .expect("Load asset utility functions failed to load");

                lua_ctx
                .load(&asset_script)
                .set_name("Load Asset Info").unwrap()
                .exec()
                .expect("Lua asset script failed!");

            });

            // let lua_load_asset_info: Function = globals.get("loadAssetInfo").expect("No function loaded!");

            // lua_load_asset_info.call::<_, ()>(()).expect("loadAssetInfo function call failed!");
        })
    }
}
