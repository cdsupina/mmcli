//! Category detection logic

use crate::models::product::ProductDetail;

/// Determine the fastener category based on product information
pub fn determine_category(product: &ProductDetail) -> String {
    let family_lower = product.family_description.to_lowercase();
    let category_lower = product.product_category.to_lowercase();
    let _detail_lower = product.detail_description.to_lowercase();
    
    // Check for specific screw head types (order matters - more specific first)
    // First check for thread forming screws (most specific)
    if family_lower.contains("thread-forming") || family_lower.contains("thread forming") {
        if family_lower.contains("button head") && family_lower.contains("screw") {
            "thread_forming_button_head_screw".to_string()
        } else if family_lower.contains("high socket head") && family_lower.contains("screw") {
            "thread_forming_high_socket_head_screw".to_string()
        } else if (family_lower.contains("low socket head") || family_lower.contains("low-profile socket head")) && family_lower.contains("screw") {
            "thread_forming_low_socket_head_screw".to_string()
        } else if family_lower.contains("socket head") && family_lower.contains("screw") {
            "thread_forming_socket_head_screw".to_string()
        } else if family_lower.contains("flat head") && family_lower.contains("screw") {
            "thread_forming_flat_head_screw".to_string()
        } else if family_lower.contains("pan head") && family_lower.contains("screw") {
            "thread_forming_pan_head_screw".to_string()
        } else if family_lower.contains("hex head") && family_lower.contains("screw") {
            "thread_forming_hex_head_screw".to_string()
        } else if family_lower.contains("screw") {
            "thread_forming_screw".to_string()
        } else {
            "thread_forming_screw".to_string()
        }
    } else if family_lower.contains("button head") && family_lower.contains("screw") {
        "button_head_screw".to_string()
    } else if family_lower.contains("high socket head") && family_lower.contains("screw") {
        "high_socket_head_screw".to_string()
    } else if (family_lower.contains("low socket head") || family_lower.contains("low-profile socket head")) && family_lower.contains("screw") {
        "low_socket_head_screw".to_string()
    } else if (family_lower.contains("ultra low socket head") || family_lower.contains("ultra low-profile socket head")) && family_lower.contains("screw") {
        "ultra_low_socket_head_screw".to_string()
    } else if family_lower.contains("standard socket head") && family_lower.contains("screw") {
        "standard_socket_head_screw".to_string()
    } else if family_lower.contains("socket head") && family_lower.contains("screw") {
        "socket_head_screw".to_string()
    } else if family_lower.contains("narrow flat head") && family_lower.contains("screw") {
        "narrow_flat_head_screw".to_string()
    } else if family_lower.contains("standard flat head") && family_lower.contains("screw") {
        "standard_flat_head_screw".to_string()
    } else if family_lower.contains("undercut flat head") && family_lower.contains("screw") {
        "undercut_flat_head_screw".to_string()
    } else if family_lower.contains("wide flat head") && family_lower.contains("screw") {
        "wide_flat_head_screw".to_string()
    } else if family_lower.contains("flat head") && family_lower.contains("screw") {
        "flat_head_screw".to_string()
    } else if family_lower.contains("pan head") && family_lower.contains("screw") {
        "pan_head_screw".to_string()
    } else if family_lower.contains("hex head") && family_lower.contains("screw") {
        "hex_head_screw".to_string()
    } else if family_lower.contains("standard oval head") && family_lower.contains("screw") {
        "standard_oval_head_screw".to_string()
    } else if family_lower.contains("undercut oval head") && family_lower.contains("screw") {
        "undercut_oval_head_screw".to_string()
    } else if family_lower.contains("oval head") && family_lower.contains("screw") {
        "oval_head_screw".to_string()
    } else if family_lower.contains("square head") && family_lower.contains("screw") {
        "square_head_screw".to_string()
    } else if family_lower.contains("binding head") && family_lower.contains("screw") {
        "binding_head_screw".to_string()
    } else if family_lower.contains("carriage head") && family_lower.contains("screw") {
        "carriage_head_screw".to_string()
    } else if family_lower.contains("cheese head") && family_lower.contains("screw") {
        "cheese_head_screw".to_string()
    } else if family_lower.contains("fillister head") && family_lower.contains("screw") {
        "fillister_head_screw".to_string()
    } else if family_lower.contains("pancake head") && family_lower.contains("screw") {
        "pancake_head_screw".to_string()
    } else if family_lower.contains("round head") && family_lower.contains("screw") {
        "round_head_screw".to_string()
    } else if family_lower.contains("truss head") && family_lower.contains("screw") {
        "truss_head_screw".to_string()
    } else if family_lower.contains("rounded head") && family_lower.contains("screw") {
        "rounded_head_screw".to_string()
    } else if family_lower.contains("12-point") && family_lower.contains("screw") {
        "12_point_head_screw".to_string()
    } else if family_lower.contains("t-handle") && family_lower.contains("screw") {
        "t_handle_screw".to_string()
    } else if family_lower.contains("t-slot") && family_lower.contains("screw") {
        "t_slot_screw".to_string()
    } else if family_lower.contains("l-handle") && family_lower.contains("screw") {
        "l_handle_screw".to_string()
    } else if family_lower.contains("domed") && family_lower.contains("screw") {
        "domed_head_screw".to_string()
    } else if family_lower.contains("headless") && family_lower.contains("screw") {
        "headless_screw".to_string()
    } else if family_lower.contains("pentagon") && family_lower.contains("screw") {
        "pentagon_head_screw".to_string()
    } else if family_lower.contains("four arm thumb") && family_lower.contains("screw") {
        "four_arm_thumb_screw".to_string()
    } else if family_lower.contains("hex thumb") && family_lower.contains("screw") {
        "hex_thumb_screw".to_string()
    } else if family_lower.contains("multilobe thumb") && family_lower.contains("screw") {
        "multilobe_thumb_screw".to_string()
    } else if family_lower.contains("rectangle thumb") && family_lower.contains("screw") {
        "rectangle_thumb_screw".to_string()
    } else if family_lower.contains("round thumb") && family_lower.contains("screw") {
        "round_thumb_screw".to_string()
    } else if family_lower.contains("spade thumb") && family_lower.contains("screw") {
        "spade_thumb_screw".to_string()
    } else if family_lower.contains("two arm thumb") && family_lower.contains("screw") {
        "two_arm_thumb_screw".to_string()
    } else if family_lower.contains("wing thumb") && family_lower.contains("screw") {
        "wing_thumb_screw".to_string()
    } else if family_lower.contains("thumb") && family_lower.contains("screw") {
        "thumb_screw".to_string()
    } else if family_lower.contains("captive panel") && family_lower.contains("screw") {
        "captive_panel_screw".to_string()
    } else if family_lower.contains("hook") && family_lower.contains("screw") {
        "hook_screw".to_string()
    } else if family_lower.contains("ring") && family_lower.contains("screw") {
        "ring_screw".to_string()
    } else if family_lower.contains("eye") && family_lower.contains("screw") {
        "eye_screw".to_string()
    } else if family_lower.contains("knob") && family_lower.contains("screw") {
        "knob_screw".to_string()
    } else if family_lower.contains("threaded") && family_lower.contains("screw") {
        "threaded_screw".to_string()
    } else if family_lower.contains("tee") && family_lower.contains("screw") {
        "tee_screw".to_string()
    } else if family_lower.contains("screw") {
        "generic_screw".to_string()
    } else if family_lower.contains("washer") {
        determine_washer_type(&family_lower)
    } else if category_lower.contains("nuts") || category_lower.contains("nut") || family_lower.contains("nut") {
        determine_nut_type(&family_lower)
    } else if family_lower.contains("unthreaded spacer") || (category_lower.contains("spacers") && family_lower.contains("spacer")) {
        determine_spacer_type(&family_lower)
    } else if category_lower.contains("standoffs") || category_lower.contains("standoff") || 
              family_lower.contains("standoff") {
        determine_standoff_type(&family_lower)
    } else if category_lower.contains("bearing") || family_lower.contains("bearing") {
        determine_bearing_type(product)
    } else if category_lower.contains("pins") || family_lower.contains("pin") {
        determine_pin_type(&family_lower)
    } else if category_lower.contains("shaft collars") || family_lower.contains("shaft collar") {
        determine_shaft_collar_type(&family_lower)
    } else if category_lower.contains("pulleys") || family_lower.contains("pulley") || family_lower.contains("sheave") {
        determine_pulley_type(&family_lower)
    } else {
        "unknown".to_string()
    }
}

/// Determine specific washer type
fn determine_washer_type(family_lower: &str) -> String {
    if family_lower.contains("cup") {
        "cup_washer".to_string()
    } else if family_lower.contains("curved") {
        "curved_washer".to_string()
    } else if family_lower.contains("dished") {
        "dished_washer".to_string()
    } else if family_lower.contains("domed") {
        "domed_washer".to_string()
    } else if family_lower.contains("double clipped") || family_lower.contains("double-clipped") {
        "double_clipped_washer".to_string()
    } else if family_lower.contains("clipped") {
        "clipped_washer".to_string()
    } else if family_lower.contains("hillside") {
        "hillside_washer".to_string()
    } else if family_lower.contains("notched") {
        "notched_washer".to_string()
    } else if family_lower.contains("perforated") {
        "perforated_washer".to_string()
    } else if family_lower.contains("pronged") {
        "pronged_washer".to_string()
    } else if family_lower.contains("rectangular") {
        "rectangular_washer".to_string()
    } else if family_lower.contains("sleeve") {
        "sleeve_washer".to_string()
    } else if family_lower.contains("slotted") {
        "slotted_washer".to_string()
    } else if family_lower.contains("spherical") {
        "spherical_washer".to_string()
    } else if family_lower.contains("split") {
        "split_washer".to_string()
    } else if family_lower.contains("square") {
        "square_washer".to_string()
    } else if family_lower.contains("tab") {
        "tab_washer".to_string()
    } else if family_lower.contains("tapered") {
        "tapered_washer".to_string()
    } else if family_lower.contains("tooth") {
        "tooth_washer".to_string()
    } else if family_lower.contains("wave") {
        "wave_washer".to_string()
    } else if family_lower.contains("wedge") {
        "wedge_washer".to_string()
    } else {
        "flat_washer".to_string() // Default to flat washer
    }
}

/// Determine specific nut type
fn determine_nut_type(family_lower: &str) -> String {
    // Locking nut sub-types (most specific first)
    if family_lower.contains("cotter pin") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "cotter_pin_locknut".to_string()
    } else if family_lower.contains("distorted thread") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "distorted_thread_locknut".to_string()
    } else if family_lower.contains("flex-top") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "flex_top_locknut".to_string()
    } else if family_lower.contains("lock washer") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "lock_washer_locknut".to_string()
    } else if family_lower.contains("nylon insert") || family_lower.contains("nylon-insert") {
        "nylon_insert_locknut".to_string()
    } else if family_lower.contains("serrations") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "serrations_locknut".to_string()
    } else if family_lower.contains("spring-stop") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "spring_stop_locknut".to_string()
    } else if family_lower.contains("steel insert") && (family_lower.contains("locknut") || family_lower.contains("lock nut")) {
        "steel_insert_locknut".to_string()
    } else if family_lower.contains("locknut") || family_lower.contains("lock nut") {
        "generic_locknut".to_string()
    } else if family_lower.contains("acorn nut") || family_lower.contains("acornnut") {
        "acorn_nut".to_string()
    } else if family_lower.contains("barrel nut") {
        "barrel_nut".to_string()
    } else if family_lower.contains("cage nut") {
        "cage_nut".to_string()
    } else if family_lower.contains("castle nut") {
        "castle_nut".to_string()
    } else if family_lower.contains("clinch nut") {
        "clinch_nut".to_string()
    } else if family_lower.contains("coupling nut") {
        "coupling_nut".to_string()
    } else if family_lower.contains("flange nut") || family_lower.contains("flangenut") {
        "flange_nut".to_string()
    } else if family_lower.contains("hex nut") || family_lower.contains("hexnut") {
        "hex_nut".to_string()
    } else if family_lower.contains("jam nut") {
        "jam_nut".to_string()
    } else if family_lower.contains("knurled thumb nut") {
        "knurled_thumb_nut".to_string()
    } else if family_lower.contains("machine screw nut") {
        "machine_screw_nut".to_string()
    } else if family_lower.contains("panel nut") {
        "panel_nut".to_string()
    } else if family_lower.contains("push on nut") || family_lower.contains("push-on nut") {
        "push_on_nut".to_string()
    } else if family_lower.contains("rivet nut") {
        "rivet_nut".to_string()
    } else if family_lower.contains("round nut") {
        "round_nut".to_string()
    } else if family_lower.contains("screw mount") {
        "screw_mount_nut".to_string()
    } else if family_lower.contains("snap in") || family_lower.contains("snap-in") {
        "snap_in_nut".to_string()
    } else if family_lower.contains("socket nut") {
        "socket_nut".to_string()
    } else if family_lower.contains("speed") {
        "speed_nut".to_string()
    } else if family_lower.contains("square") {
        "square_nut".to_string()
    } else if family_lower.contains("tamper resistant") || family_lower.contains("tamper-resistant") {
        "tamper_resistant_nut".to_string()
    } else if family_lower.contains("threadless") {
        "threadless_nut".to_string()
    } else if family_lower.contains("thumb") {
        "thumb_nut".to_string()
    } else if family_lower.contains("tube end") {
        "tube_end_nut".to_string()
    } else if family_lower.contains("twist close") || family_lower.contains("twist-close") {
        "twist_close_nut".to_string()
    } else if family_lower.contains("weld") {
        "weld_nut".to_string()
    } else if family_lower.contains("with pilot hole") {
        "with_pilot_hole_nut".to_string()
    } else if family_lower.contains("wing nut") || family_lower.contains("wingnut") {
        "wing_nut".to_string()
    } else if family_lower.contains("cap nut") || family_lower.contains("capnut") {
        "cap_nut".to_string()
    } else {
        "generic_nut".to_string()
    }
}

/// Determine specific standoff type
fn determine_standoff_type(family_lower: &str) -> String {
    if family_lower.contains("male-female") || family_lower.contains("male female") {
        "male_female_hex_standoff".to_string()
    } else if family_lower.contains("female") && family_lower.contains("threaded") {
        "female_hex_standoff".to_string()
    } else {
        "generic_standoff".to_string()
    }
}

/// Determine specific spacer type
fn determine_spacer_type(family_lower: &str) -> String {
    if family_lower.contains("aluminum") {
        "aluminum_unthreaded_spacer".to_string()
    } else if family_lower.contains("stainless steel") || family_lower.contains("18-8") || family_lower.contains("316") {
        "stainless_steel_unthreaded_spacer".to_string()
    } else if family_lower.contains("nylon") {
        "nylon_unthreaded_spacer".to_string()
    } else {
        "unthreaded_spacer".to_string()
    }
}

/// Determine specific pin type
fn determine_pin_type(family_lower: &str) -> String {
    if family_lower.contains("clevis pin with retaining ring groove") {
        "clevis_pin_with_retaining_ring_groove".to_string()
    } else if family_lower.contains("clevis pin") {
        "clevis_pin".to_string()
    } else {
        "generic_pin".to_string()
    }
}

/// Determine specific shaft collar type
fn determine_shaft_collar_type(family_lower: &str) -> String {
    if family_lower.contains("face-mount shaft collar") || family_lower.contains("face mount shaft collar") {
        "face_mount_shaft_collar".to_string()
    } else if family_lower.contains("flange-mount shaft collar") || family_lower.contains("flange mount shaft collar") {
        "flange_mount_shaft_collar".to_string()
    } else {
        "generic_shaft_collar".to_string()
    }
}

/// Determine specific bearing type
fn determine_bearing_type(product: &ProductDetail) -> String {
    let family_lower = product.family_description.to_lowercase();
    let category_lower = product.product_category.to_lowercase();
    let plain_type = product.specifications.iter()
        .find(|s| s.attribute.eq_ignore_ascii_case("Plain Bearing Type"))
        .and_then(|s| s.values.first())
        .map(|v| v.as_str())
        .unwrap_or("");
    
    // Check for mounted bearings first (more specific)
    if family_lower.contains("mounted") || category_lower.contains("mounted") {
        let mount_type = product.specifications.iter()
            .find(|s| s.attribute.eq_ignore_ascii_case("Mounted Bearing Type"))
            .and_then(|s| s.values.first())
            .map(|v| v.to_lowercase())
            .unwrap_or_default();
        
        let description_lower = product.detail_description.to_lowercase();
        
        if mount_type.contains("two-bolt flange") || mount_type.contains("flange") {
            if family_lower.contains("low-profile") || family_lower.contains("low profile") 
               || description_lower.contains("low-profile") || description_lower.contains("low profile") {
                "low_profile_flange_mounted_ball_bearing".to_string()
            } else {
                "flange_mounted_ball_bearing".to_string()
            }
        } else if mount_type.contains("pillow") {
            "pillow_block_mounted_ball_bearing".to_string()
        } else {
            "generic_mounted_bearing".to_string()
        }
    } else if family_lower.contains("flanged") || plain_type.eq_ignore_ascii_case("Flanged") {
        if family_lower.contains("sleeve") || family_lower.contains("plain") {
            "flanged_sleeve_bearing".to_string()
        } else {
            "flanged_bearing".to_string()
        }
    } else if family_lower.contains("sleeve") || family_lower.contains("plain") {
        "sleeve_bearing".to_string()
    } else if family_lower.contains("ball") {
        "ball_bearing".to_string()
    } else if family_lower.contains("linear") {
        "linear_bearing".to_string()
    } else if family_lower.contains("needle") {
        "needle_bearing".to_string()
    } else if family_lower.contains("roller") {
        "roller_bearing".to_string()
    } else {
        "generic_bearing".to_string()
    }
}

/// Determine specific pulley type
fn determine_pulley_type(family_lower: &str) -> String {
    if family_lower.contains("wire rope") {
        "wire_rope_pulley".to_string()
    } else if family_lower.contains("rope") && !family_lower.contains("wire") {
        "rope_pulley".to_string()
    } else if family_lower.contains("v-belt") || family_lower.contains("belt") {
        "v_belt_pulley".to_string()
    } else if family_lower.contains("sheave") {
        "sheave".to_string()
    } else {
        "pulley".to_string()
    }
}