-- Noise Parameters
dividend = 0.10
-- Generation Parameters
max_build_height = 16 * 16
max_gen_height = 90 -- Max 250
min_gen_height = 60 -- Min 10

-- Flora Parameters 
tree_prob = 500 -- 1 in treeProb
tree_log_min_height = 4
tree_log_max_height = 8
grass_prob = 50 -- 1 in grassProb
rose_prob = 50 -- 1 in roseProb

-- Get Needed IDs
air_id = get_id_by_name("Air")
bedrock_id = get_id_by_name("Bedrock")
stone_id = get_id_by_name("Stone")
dirt_id = get_id_by_name("Dirt")
grass_id = get_id_by_name("Grass")
log_id = get_id_by_name("Log")
leaves_id = get_id_by_name("Leaves")
topGrass_id = get_id_by_name("TopGrass")
rose_id = get_id_by_name("Rose")



for x = 0, 15, 1
do
    for z = 0, 15, 1
    do
        noise_val = get_noise_2d("OpenSimplex2", column_x / dividend + (x / 16.0) / dividend, column_z / dividend + (z / 16.0) / dividend)
        noise_val = (noise_val + 1.0) / 2.0;

        height_offset = math.floor((max_gen_height - 1 - min_gen_height) * noise_val)

        top_height = min_gen_height + height_offset

        -- Terrain Generation
        set_layers(0, 0, bedrock_id)
    end
end