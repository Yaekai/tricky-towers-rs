# Staged assets the code never uses (checked 2026-10-09)

Method: every key in the staged manifest (manifest.ron, 2,823 entries) was looked up as a
string literal in src/ and tests/, and by the prefix of any format! name. 622 keys have no
match. This is a text search, so a few may be built in pieces at run time; each row says
what to check. Staged art stays local and is not in the repo; this list is for the PC thread.

## Likely the cause of open items
- bg_race_cloud_var02_0, var03, var05, var06_0, var08_0, var09, var10_0; bg_race_cloud_large_race_endless_01/02: the title's back cloud banks (Max's menu screenshots). Add to compose_menu.ps1 or the title scene.
- bg_race_waterline01..04: the water surface line on the title and menus.
- bg_puzzle_sand_storm, bg_puzzle_foreground_sand, bg_puzzle_pink_sand: the Puzzle bottom sandstorm band.
- trophy_online: the trophy in the Online Battle picture (the earth-box issue).
- fx_cast_effect* (animals, blobs, sines, circles): the Spirit Animal and Indie wizards' casts; skin beams still use wizard 1's.
- music_win_game_loop, music_lose_game_loop: the results music, not played.
- hud_next_brick_* (background and shapes i, j, l, o, s, t, z): the next-brick card art; the card uses next_brick_* icons instead.
- fx_ivy_<shape>_0..4: per-shape Ivy growth frames; Ivy uses vine_overlay.
- fx_skull01_head/jaw, fx_skull02_head: Ghost skulls.
- race_pro_island_2..7_lit, race_pro_cloud: the Race Pro background pieces' bright copies (the flash).
- rival_brick_I..Z, rival_minibase_L/R: trial rival tower bricks.
- menu_rank_* and menu_stats_popup_*: rank bars and the Stats popup art; those screens are built without them.
- menu_cup_*_small, menu_cup_online_*, menu_cup_super_small: cup icons for Big/Super/Online.

## Probably not used yet (no feature needs them)
- fx_moss_* (moss on bricks), fx_grass_large_*, fx_house01_grass.
- hud_spell_super, hud_spell_background_super(_duper), spell_orb_light, survival_star.
- confetti_1/2, explosion_debris, explosion_puff, cast_circle_*, cast_spark, cast_sunrays.
- bg_race_cactus_*, bg_race_checkpoint_flag(_pole), bg_race_finish_line, bg_race_sun.
- bg_race_snow_var01..03, bg_race_galaxy_background (the Galaxy field is staged as brick_galaxy_background).
- fonts: font_enzo_black, font_hans_sans_*, font_meiryo_*, font_myriad (other languages).
- text_ar, text_ch, text_de ... text_tw: other language letter sheets (the language list is by name).
- menu_pad_*, menu_*_xbox, menu_*_psx, menu_*_button_large_*: controller glyph variants; the pad picks by vendor, check the lookup.
- menu_key_c, menu_key_tab, menu_keyboard_arrows, menu_demo_message, menu_host_dropped, menu_made_with_unity_logo: UI hints and credits, not drawn.
- match_spotlight_winner_world01..03, match_star_outline.
- water_band_puzzle, water_band_race, water_band_survival, water_pro: the mode water bands; check the water code before using.
- misc_* and pacer_layer, page_dot_off, menu_pagination_active, menu_gradient_texture_font05.

## Sounds (328 keys)
- sfx_voice_* (210): per-character voice lines. The voice picker may build names in pieces; check the voice lookup in art.rs before treating them as unused.
- sfx_piano (26), sfx_4th_place_* (6), sfx_auto_rotate_*, sfx_autobrick_hit_*, sfx_brick_drop_sand_*, sfx_coin_*, sfx_fireworks, sfx_medal, sfx_petrify, sfx_skeleton, sfx_xp, sfx_spell, sfx_grass, sfx_ice, sfx_amb_thunderstrikes_*: events the original plays and we don't yet.

## Next step
Wire the first group (title clouds and waterline, Puzzle sandstorm, trophy_online, cast effects,
win/lose music, next-brick card art, Ivy frames, skulls, Race Pro lit pieces, rival bricks,
rank and stats art) one by one, each checked against the install's readout.
