#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct Scene {
    view: vec4<f32>,
    player: vec4<f32>,
    anchor: vec4<f32>,
    clock: vec4<f32>,
    bounds: vec4<f32>,
    counts: vec4<f32>,
    controls: vec4<f32>,
    motion: vec4<f32>,
    appearance: vec4<f32>,
    feedback: vec4<f32>,
    journey: vec4<f32>,
    tools: vec4<f32>,
    placement: vec4<f32>,
    bell: vec4<f32>,
    finds: array<vec4<f32>, 16>,
    roots: array<vec4<f32>, 108>,
    root_widths: array<vec4<f32>, 108>,
    tendrils: array<vec4<f32>, 56>,
    props: array<vec4<f32>, 32>,
    rooms: array<vec4<f32>, 32>,
    room_state: array<vec4<f32>, 32>,
    gates: array<vec4<f32>, 24>,
    gate_state: array<vec4<f32>, 24>,
    sites: array<vec4<f32>, 24>,
    creatures: array<vec4<f32>, 24>,
    creature_state: array<vec4<f32>, 24>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> scene: Scene;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var map_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var map_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(3) var ground_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var ground_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var traveller_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var traveller_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var hauler_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var hauler_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var terrain_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var terrain_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(11) var ecosystem_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(12) var ecosystem_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(13) var vertical_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(14) var vertical_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(15) var idle_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(16) var idle_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(17) var objects_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(18) var objects_sampler: sampler;
// Sprite positions are ground contacts, not the centre of an upright body.
fn sprite_uv(p: vec2<f32>, foot: vec2<f32>, size: vec2<f32>, flip: f32) -> vec2<f32> {
    let q = (p-foot)/size;
    return vec2(q.x*flip+0.5,0.9375-q.y);
}
fn within(uv: vec2<f32>) -> bool {
    return all(uv >= vec2(0.002)) && all(uv <= vec2(0.998));
}
fn atlas_uv(uv: vec2<f32>, frame: f32, columns: f32, rows: f32) -> vec2<f32> {
    return (uv+vec2(frame%columns,floor(frame/columns)))/vec2(columns,rows);
}
// Forty-eight opaque poses per gait; no translucent silhouette interpolation.
fn animate(art_texture: texture_2d<f32>, art_sampler: sampler,
           uv: vec2<f32>, phase: f32, first: f32, rows: f32) -> vec4<f32> {
    // Sample one complete pose. Alpha cross-fades create translucent extra
    // legs even when the two poses are only a fraction of a frame apart.
    let frame=first+floor(phase)%48.0;
    return textureSampleLevel(art_texture,art_sampler,atlas_uv(uv,frame,8.0,rows),0.0);
}
fn prop_sample(p: vec2<f32>, prop: vec4<f32>) -> vec4<f32> {
    let uv = sprite_uv(p,prop.xy,vec2(prop.z),select(1.0,-1.0,fract(prop.w)>0.1));
    if !within(uv) { return vec4(0.0); }
    if prop.w>=4.0 {return textureSampleLevel(objects_texture,objects_sampler,atlas_uv(uv,floor(prop.w)-4.0,4.0,3.0),0.0);}
    return textureSampleLevel(terrain_texture,terrain_sampler,atlas_uv(uv,floor(prop.w),2.0,2.0),0.0);
}

const TAU: f32 = 6.28318530718;

fn hash(p: vec2<f32>) -> f32 {
    var q = fract(vec3<f32>(p.x, p.y, p.x) * 0.1031);
    q += dot(q, q.yzx + 33.33);
    return fract((q.x + q.y) * q.z);
}
fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}
// Each grid corner owns a stable, rotated photographic sample. Blending at
// constant world scale hides repetition without stretching or mirroring rock.
fn surface_sample(p:vec2<f32>, id:vec2<f32>, region:f32)->f32 {
    let h=hash(id);
    var q=p/215.0;
    if h<0.25 {q=vec2(-q.y,q.x);} else if h<0.5 {q=-q;} else if h<0.75 {q=vec2(q.y,-q.x);}
    let uv=clamp(fract(q+vec2(h,hash(id+17.0))*19.0),vec2(0.001),vec2(0.999));
    return textureSampleLevel(ground_texture,ground_sampler,atlas_uv(uv,floor(region+0.1),2.0,2.0),0.0).r;
}
fn surface(p:vec2<f32>,region:f32)->f32 {
    let grid=p/185.0;let id=floor(grid);let f=fract(grid);let w=f*f*(3.0-2.0*f);
    return mix(mix(surface_sample(p,id,region),surface_sample(p,id+vec2(1.0,0.0),region),w.x),
        mix(surface_sample(p,id+vec2(0.0,1.0),region),surface_sample(p,id+vec2(1.0),region),w.x),w.y);
}
fn field(p: vec2<f32>) -> f32 {
    let uv = (p - scene.bounds.xy) / scene.bounds.zw;
    return textureSampleLevel(map_texture, map_sampler, uv, 0.0).r * 512.0 - 256.0;
}
fn line(d: f32, width: f32) -> f32 {
    return 1.0 - smoothstep(width, width + 1.3, abs(d));
}
fn segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let h = clamp(dot(p - a, ab) / max(dot(ab, ab), 0.01), 0.0, 1.0);
    return length(p - a - ab * h);
}
fn shadow(p: vec2<f32>, source: vec2<f32>) -> f32 {
    let ray = p - source;
    var shade = 1.0;
    for (var i = 1; i <= 9; i += 1) {
        let t = f32(i) / 10.0;
        let gap = field(source + ray * t);
        shade = min(shade, smoothstep(-3.0, 12.0 + t * 9.0, gap));
    }
    return shade;
}
fn lamp(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    let d = length(p - center) / radius;
    return exp(-d * d * 2.7);
}
fn fbm(p: vec2<f32>) -> f32 {
    return noise(p) * 0.56 + noise(p * 2.07 + 17.0) * 0.28 + noise(p * 4.13 - 9.0) * 0.16;
}
fn solid(d: f32) -> f32 { return 1.0 - smoothstep(-0.7, 1.2, d); }
fn ellipse(p: vec2<f32>, size: vec2<f32>) -> f32 {
    return (length(p / size) - 1.0) * min(size.x, size.y);
}
fn palette(region: f32) -> vec3<f32> {
    return vec3(select(select(0.38,0.68,region>1.5),0.30,region>2.5));
}
fn gate_mark(q: vec2<f32>, kind: f32) -> f32 {
    if kind<0.5 { return line(length(q)-7.0,0.8)+line(q.x,0.6)*(1.0-smoothstep(8.0,11.0,abs(q.y)))+line(q.y,0.6)*(1.0-smoothstep(8.0,11.0,abs(q.x))); }
    if kind<1.5 { return line(abs(q.x)-abs(q.y)*0.55-2.0,0.8)*(1.0-smoothstep(7.0,10.0,abs(q.y))); }
    if kind<2.5 { return line(length(q-vec2(0.0,3.0))-7.0,1.1)*step(q.y,5.0)+line(q.x,0.9)*step(abs(q.y+4.0),6.0); }
    if kind<3.5 { return line(abs(q.x+sin(q.y*0.3))-5.0,0.7)*step(abs(q.y),10.0)+line(q.x-sin(q.y*0.3),0.7)*step(abs(q.y),10.0); }
    return line(length(q-vec2(-8.0,0.0))-2.5,0.6)+line(length(q)-2.5,0.6)+line(length(q-vec2(8.0,0.0))-2.5,0.6);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let p = scene.view.xy + (uv - 0.5) * vec2(scene.view.z, -scene.view.w);
    let time = scene.clock.x;
    let data = textureSampleLevel(map_texture, map_sampler, (p - scene.bounds.xy) / scene.bounds.zw, 0.0);
    let room_id = min(u32(round(data.b * 255.0)), 31u);
    let room = scene.rooms[room_id];
    let region = scene.room_state[room_id].x;
    let remembered = scene.room_state[room_id].y;
    let tone = palette(region);
    let d = data.r * 512.0 - 256.0;
    let rough = noise(p * 0.033) * 3.7 + noise(p * 0.11) * 1.6;
    let rock_edge = d + rough - 1.6;
    let inside = smoothstep(-0.5, 1.2, rock_edge);
    let dist_player = distance(p, scene.player.xy);
    if scene.controls.x > 0.5 {
        let pixel = scene.view.w / 760.0;
        let rim = exp(-abs(d) / (pixel * 1.7));
        var memory = tone * remembered * (rim * 0.19 + inside * 0.009);
        var marker=0u;
        for (var i = 0u; i < u32(scene.counts.z); i += 1u) {
            let s = scene.sites[i];
            let site_d = distance(p, s.xy);
            if s.z < 0.5 {
                memory += vec3(0.23, 0.34, 0.30) * line(site_d / pixel - 4.0, 0.5) * remembered;
            } else if s.z > 2.5 && s.z < 3.5 {
                let charted=f32((u32(scene.tools.w)>>marker)&1u);
                memory += vec3(0.31, 0.33, 0.25) * line(site_d / pixel - 5.0, 0.7) * max(remembered,charted) * (0.55 + s.w * 0.45);
                marker+=1u;
            }
        }
        for (var i=0u;i<u32(scene.counts.y);i+=1u) {
            let g=scene.gates[i];
            let gid=min(u32(round(textureSampleLevel(map_texture,map_sampler,(g.xy-scene.bounds.xy)/scene.bounds.zw,0.0).b*255.0)),31u);
            if scene.room_state[gid].y>0.5 {
                memory+=vec3(gate_mark((p-g.xy)/(pixel*0.65),scene.gate_state[i].z)*0.23);
            }
        }
        memory += vec3(0.63, 0.73, 0.61) * exp(-dist_player * dist_player / (pixel * pixel * 4.0));
        return vec4(vec3(dot(memory, vec3(0.333333)) + 0.001), 1.0);
    }
    let vertical=abs(scene.appearance.y)>abs(scene.appearance.x)*1.2;
    let phase=scene.motion.x/76.0*48.0;
    let body_bob=-1.0*cos(floor(phase)/48.0*TAU*2.0);
    let idle_sway=sin(time*0.73)*0.012*scene.feedback.w;
    let placing_bend=select(0.0,sin(clamp(1.0-scene.tools.y/1.15,0.0,1.0)*3.14159265)*0.09,scene.tools.y>0.0);
    let idle_breath=sin(time*1.45)*0.005*scene.feedback.w-placing_bend;
    // These anchors are measured on each painted upper-body layer. The lamp
    // pixels, halo and world illumination use the same animated transform.
    let lamp_pixel=select(vec2(129.0,111.0),select(vec2(143.0,119.0),vec2(87.0,102.0),scene.appearance.y<0.0),vertical);
    let lamp_flip=select(scene.motion.y,1.0,vertical);
    let lamp_weight=clamp((0.9-lamp_pixel.y/256.0)/0.6,0.0,1.0);
    let lantern=scene.player.xy+vec2((lamp_pixel.x/192.0-0.5+idle_sway*lamp_weight)*45.0*lamp_flip,(0.9375-(lamp_pixel.y+body_bob)/256.0+idle_breath*lamp_weight)*60.0-scene.motion.z*0.45);
    let flicker=0.95+0.025*sin(time*7.1)+0.017*sin(time*11.73+1.8)+0.018*sin(time*2.37);
    let lantern_tint=vec3(1.19,0.91,0.60);
    let brightness = clamp(scene.player.z, 0.0, 1.0);
    let radius = 22.0 + 95.0 * brightness;
    let pulse = clamp(scene.player.w / 1.65, 0.0, 1.0);
    let pulse_radius = 45.0 + 430.0 * sqrt(max(0.0, 1.0 - pulse));
    let pulse_light = pulse * (1.0 - smoothstep(pulse_radius - 65.0, pulse_radius + 45.0, dist_player)) * 0.78;
    let personal = lamp(p, lantern, radius) * (0.004 + brightness * 0.24) * flicker;
    // The lamp is elevated above its floor contact. A painted upright wall
    // must not swallow its rays when the lamp overlaps it on screen.
    let occlusion = shadow(p, scene.player.xy);
    var illumination = (personal + pulse_light) * occlusion;
    let ending = scene.controls.w;
    if ending > 0.0 {
        let reveal = smoothstep(0.4, 3.5, ending) * (1.0 - smoothstep(5.2, 9.5, ending));
        illumination += lamp(p, scene.player.xy, 1050.0) * reveal * 0.62;
    }
    let placed_flame=scene.anchor.xy+vec2(0.0,10.8);
    let burn=smoothstep(0.0,4.0,scene.tools.x)*(1.0-scene.tools.z*0.25*(0.6+0.4*sin(time*29.0)));
    let anchor_light = lamp(p, placed_flame, 230.0) * scene.anchor.z * 0.58*flicker*burn;
    var amber_light = anchor_light;
    if scene.anchor.z > 0.5 && anchor_light > 0.001 {
        amber_light *= shadow(p, scene.anchor.xy);
    }

    // Light-bearing growths give rooms landmarks without exposing their exits.
    for (var i = 0u; i < u32(scene.counts.z); i += 1u) {
        let site = scene.sites[i];
        let ds = distance(p, site.xy);
        if site.z < 0.5 {
            illumination += exp(-ds * ds / 15000.0) * 0.24;
        } else if site.z < 2.5 && site.w < 0.5 {
            illumination += exp(-ds * ds / 4600.0) * 0.10;
        } else if site.z > 2.5 && site.z < 3.5 && site.w > 0.5 {
            illumination += exp(-ds * ds / 18500.0) * 0.16;
        }
    }
    let light = illumination + amber_light;
    // Black material against diffuse grey illumination. No emissive wall rims.
    let n = fbm(p * vec2(0.012, 0.017));
    let fine = noise(p * 0.22);
    let fog = fbm(p * 0.006 + vec2(time * 0.009, -time * 0.004));
    let material=surface(p,region);
    let ground=(0.07+pow(max(material,0.0),0.60)*select(0.52,0.37,region>2.5)+n*0.025)*inside;
    var color = vec3(0.0012 + ground * light);
    color += ground * (personal * occlusion+amber_light) * (lantern_tint - vec3(1.0));
    if region>1.5 && region<2.5 {
        let ring=line(dist_player-(1.0-scene.feedback.z)*95.0,0.25)*scene.feedback.z*scene.feedback.z;
        color+=vec3(ring*(0.0008+n*0.0015))*inside;
    }
    let gradient = vec2(field(p + vec2(4.0, 0.0)) - field(p - vec2(4.0, 0.0)),
                        field(p + vec2(0.0, 4.0)) - field(p - vec2(0.0, 4.0)));
    let normal = normalize(gradient + vec2(0.0001));
    let to_light = normalize(scene.player.xy - p + vec2(0.001));
    let facing = max(0.0, dot(normal, to_light));
    let ledge = (1.0 - inside) * exp(min(rock_edge, 0.0) * 0.10);
    color += vec3(ledge * (0.06 + n * 0.09) * light * facing);
    color += vec3(remembered * exp(-abs(d) * 0.20) * 0.0011);

    color += vec3(pow(fog,3.0)*light*inside*0.016);
    // Tapered branches pull back into their wall attachment. These are the
    // same moving segments used for root drag; no stationary stripe overlay.
    var root_distance=1000.0;
    for(var i=0u;i<u32(scene.appearance.w);i+=1u) {
        let r=scene.roots[i];let widths=scene.root_widths[i].xy;
        if all(abs(p-(r.xy+r.zw)*0.5)<abs(r.zw-r.xy)*0.5+vec2(max(widths.x,widths.y)+2.0)) {
            let t=clamp(dot(p-r.xy,r.zw-r.xy)/max(dot(r.zw-r.xy,r.zw-r.xy),0.01),0.0,1.0);
            root_distance=min(root_distance,segment(p,r.xy,r.zw)-mix(widths.x,widths.y,t));
        }
    }
    let root_alpha=solid(root_distance)*inside;
    color=mix(color,vec3(0.0007+light*(0.018+fine*0.018)),root_alpha);
    color+=vec3(exp(-abs(root_distance+1.0)*2.0)*light*0.018)*inside;
    // Painted ledges remain solid silhouettes. Details are lit by the same
    // local sources; opaque atlas rectangles are never composited.
    for (var i=0u;i<u32(scene.motion.w);i+=1u) {
        let prop=scene.props[i];
        let art=prop_sample(p,prop);
        let surface=pow(max(art.r,0.0),0.65);
        let face_light=light+(personal+pulse_light)*0.18;
        color=mix(color,vec3(surface*face_light*0.76+0.001),art.a);
    }

    for(var i=0u;i<16u;i+=1u) {
        let find=scene.finds[i];let q=p-find.xy;
        if find.z>0.5 && all(abs(q)<vec2(45.0,80.0)) {
            if find.z>2.5 {
                let art=prop_sample(p,vec4(find.xy,62.0,4.0));
                color=mix(color,vec3(pow(max(art.r,0.0),0.65)*(light*0.8+0.003)),art.a);
                for(var j=0;j<3;j+=1) {
                    let hole=q-vec2((f32(j)-1.0)*5.0,34.0);
                    color+=vec3(line(length(hole)-1.4,0.35)*(0.002+light*0.035)*(1.0-find.w*0.6));
                }
            } else {
                let art=prop_sample(p,vec4(find.xy,22.0,select(12.0,13.0,find.z>1.5)));
                color=mix(color,vec3(pow(max(art.r,0.0),0.60)*(light*0.85+0.006)),art.a);
            }
        }
    }
    if scene.bell.z>0.5 {
        let art=prop_sample(p,vec4(scene.bell.xy,22.0,12.0));
        color=mix(color,vec3(pow(max(art.r,0.0),0.60)*(light*0.85+0.006)),art.a);
    }

    // Passage families have different construction and silhouette, not colours.
    for (var i = 0u; i < u32(scene.counts.y); i += 1u) {
        let g = scene.gates[i];
        let state = scene.gate_state[i];
        let v = p - g.xy;
        let across = dot(v, g.zw);
        let along = dot(v, vec2(-g.w, g.z));
        if abs(along)<state.x+18.0 && abs(across)<50.0 {
            let gap=smoothstep(state.x*state.y-4.0,state.x*state.y+3.0,abs(along));
            var body=0.0;
            var detail=0.0;
            if state.z<0.5 {
                let iris=ellipse(vec2(across,along),vec2(22.0,state.x));
                body=solid(iris)*gap;
                detail=line(iris,1.2)+pow(abs(cos(atan2(along/state.x,across/22.0)*6.0)),20.0)*body;
            } else if state.z<1.5 {
                let thorn=across-sin(along*0.05+time*0.3)*7.0;
                let branch=abs(thorn)-fract((along+state.x)/23.0)*27.0;
                body=max(line(thorn,4.0),line(branch,3.0)*(1.0-smoothstep(13.0,28.0,abs(thorn))))*gap;
                detail=line(branch+2.0,0.5)*gap*0.35;
            } else if state.z<2.5 {
                let ribs=abs(across)-(8.0+13.0*cos(along/state.x*1.57))+noise(vec2(along*0.15,across*0.09))*2.0;
                let joints=line(abs(fract(along/18.0)-0.5)*18.0,1.1)*step(abs(across),18.0);
                body=max(line(ribs,5.5),joints*0.65)*gap;
                detail=line(ribs+2.5,0.6)*gap*0.6;
            } else if state.z<3.5 {
                let threads=abs(fract((along+sin(across*0.10+time*0.5)*3.0)/9.0)-0.5)*9.0;
                let cloth_uv=vec2(across/100.0+0.5,along/(state.x*2.0)+0.5);
                let cloth=textureSampleLevel(terrain_texture,terrain_sampler,atlas_uv(cloth_uv,3.0,2.0,2.0),0.0);
                body=max((1.0-smoothstep(0.5,1.3,threads))*0.35,cloth.a*0.6)*gap;
                detail=(cloth.r*cloth.a*0.55+line(across-sin(along*0.12)*3.0,0.5)*0.25)*gap;
            } else {
                body=step(abs(across),13.0)*gap;
                detail=line(abs(across)-13.0,1.1)*gap;
                for(var socket=0;socket<3;socket+=1) {
                    let sq=vec2(across,along-(f32(socket)-1.0)*state.x*0.6);
                    detail+=line(length(sq)-8.0,1.1);
                    color+=lantern_tint*exp(-dot(sq,sq)/14.0)*step(f32(socket)+0.5,scene.journey.x)*0.28;
                }
            }
            color=mix(color,vec3(0.002+light*(0.045+noise(p*0.24)*0.09)),body*(1.0-state.y*0.8));
            color+=vec3(detail*(light*0.17+0.002))*(1.0-state.y*0.6);
            color+=vec3(gate_mark(vec2(across,along),state.z)*(light*0.15+0.008)*gap);
        }
    }

    for (var i = 0u; i < u32(scene.counts.z); i += 1u) {
        let s = scene.sites[i];
        let v = p - s.xy;
        let ds = length(v);
        let a = atan2(v.y, v.x);
        var form = 0.0;
        var tint = vec3(0.35, 0.46, 0.42);
        var glow = 0.0;
        if s.z < 0.5 {
            let hollow=distance(s.xy,scene.rooms[3].xy)<10.0;
            var wood=vec4(0.0);
            if hollow {wood=prop_sample(p,vec4(s.xy-vec2(0.0,12.0),108.0,11.0));}
            else {
                let wood_uv=sprite_uv(p,s.xy-vec2(0.0,12.0),vec2(100.0,74.0),1.0);
                if within(wood_uv) {wood=textureSampleLevel(terrain_texture,terrain_sampler,atlas_uv(wood_uv,0.0,2.0,2.0),0.0);}
            }
            color=mix(color,vec3(pow(max(wood.r,0.0),0.65)*(light*0.72+0.016)),wood.a);
            form=exp(-dot(v-vec2(1.0,14.0),v-vec2(1.0,14.0))/2.8);
            glow=0.28+0.015*sin(time*0.6);
            if hollow {
                for(var socket=0;socket<3;socket+=1) {
                    let sq=v-vec2((f32(socket)-1.0)*17.0,29.0);
                    color*=1.0-exp(-dot(sq,sq)/16.0)*0.7;
                    color+=vec3(line(length(sq)-5.0,0.45)*(0.008+light*0.10));
                    color+=lantern_tint*exp(-dot(sq,sq)/5.0)*step(f32(socket)+0.5,scene.journey.x)*0.32;
                }
            }
        } else if s.z < 2.5 {
            let folded = ellipse(v, vec2(13.0, 29.0)) + sin(v.y * 0.22) * 1.6;
            color *= 1.0 - solid(folded) * 0.9;
            let split = abs(v.x + sin(v.y * 0.065) * 3.0);
            form = exp(-split * split * 0.5) * (1.0 - smoothstep(7.0, 22.0, abs(v.y)));
            glow = (1.0 - s.w) * 0.48;
            form+=gate_mark(v,select(2.0,3.0,s.z>1.5))*0.7*(1.0-s.w);
        } else if s.z < 3.5 {
            let ad = distance(scene.anchor.xy, s.xy);
            let indirect = scene.anchor.z * smoothstep(85.0, 95.0, ad) * (1.0 - smoothstep(210.0, 220.0, ad)) * (1.0 - smoothstep(0.07, 0.12, brightness));
            let opening = max(s.w, select(0.0, indirect, s.z > 3.1));
            var body = 0.0;
            for (var k = 0; k < 7; k += 1) {
                let fk = f32(k) - 3.0;
                let off = fk * (5.5 + opening * 3.5);
                let shard = max(abs(v.x - off - sin(v.y * 0.018 + fk) * 3.0) - 3.9, abs(v.y + fk * 3.0) - (46.0 - abs(fk) * 5.0));
                body = max(body, solid(shard));
                form += line(shard, 0.1) * (0.1 + opening * 0.3);
            }
            color *= 1.0 - body * 0.93;
            form += exp(-v.x * v.x / (1.0 + opening * 22.0)) * exp(-v.y * v.y / 620.0) * opening;
            glow = s.w * 0.30;
        } else if s.z < 4.5 {
            form = exp(-ds * ds / 5.0);
            glow = (1.0 - s.w) * 0.35;
        } else {
            let cleft = abs(v.x) - (4.0 + sin(v.y * 0.027) * 3.0);
            let upright = (1.0 - smoothstep(57.0, 79.0, abs(v.y)));
            color *= 1.0 - solid(cleft - 15.0) * upright * 0.92;
            form = exp(-abs(cleft) * 0.7) * upright * 0.2;
            glow = s.w * 0.9;
        }
        color += vec3(form * (light * 0.28 + glow));
        color += vec3(exp(-ds * ds / 120.0) * glow * 0.035);
    }

    // Animation advances with real travel, so a creature caught in light or
    // blocked by rock cannot skate or walk in place. Separate silhouette families.
    for (var i=0u;i<u32(scene.counts.w);i+=1u) {
        let c=scene.creatures[i]; let state=scene.creature_state[i];
        var size=vec2(116.0,116.0);
        if c.z>0.5 && c.z<1.5 {size=vec2(98.0,151.0);}
        if c.z>1.5 && c.z<2.5 {size=vec2(94.0,72.0);}
        let uv=sprite_uv(p,c.xy,size,state.w);
        if within(uv) {
            var art=vec4(0.0);
            if c.z<0.5 {
                let phase=state.z/104.0*48.0;
                art=animate(hauler_texture,hauler_sampler,uv,phase,0.0,6.0);
            } else {
                art=animate(ecosystem_texture,ecosystem_sampler,uv,state.z/68.0*48.0,select(0.0,48.0,c.z>1.5),12.0);
            }
            let lit=pow(max(art.r,0.0),0.65)*(light+personal*0.05)*0.66;
            color=mix(color,vec3(lit),art.a*clamp((light+personal*0.03)*28.0,0.0,1.0));
        }
    }

    var shadow_distance=10000.0;
    if scene.journey.w>0.5 {
        for(var i=0u;i<56u;i+=1u) {
            let limb=scene.tendrils[i];
            let j=f32(i%8u);
            let t0=1.0-j/8.0;
            let t1=max(0.0,1.0-(j+1.0)/8.0);
            let w0=0.3+68.0*t0*t0;
            if all(abs(p-(limb.xy+limb.zw)*0.5)<abs(limb.zw-limb.xy)*0.5+vec2(w0+35.0)) {
                let w1=0.3+68.0*t1*t1;
                let along=clamp(dot(p-limb.xy,limb.zw-limb.xy)/max(dot(limb.zw-limb.xy,limb.zw-limb.xy),0.01),0.0,1.0);
                let width=mix(w0,w1,along);
                shadow_distance=min(shadow_distance,segment(p,limb.xy,limb.zw)-width);
            }
        }
    }
    let shadow_fray=shadow_distance+(fbm(p*0.043+time*0.015)-0.5)*5.0;
    let shadow_body=1.0-smoothstep(-1.0,1.0,shadow_fray);
    let shadow_rim=exp(-abs(shadow_fray)/7.0);
    color=mix(color,vec3(0.0002),shadow_body);
    color+=vec3(shadow_rim*(pulse_light*0.035+personal*0.035));

    // A handful of particulate flecks drift slowly through illuminated air.
    let drift_p = p + vec2(time * 1.2, -time * 2.3);
    let cell = floor(drift_p / 47.0);
    let offset = vec2(hash(cell), hash(cell + 13.7)) * 37.0 + 5.0;
    let speck = exp(-dot(fract(drift_p / 47.0) * 47.0 - offset, fract(drift_p / 47.0) * 47.0 - offset) * 1.8);
    color += vec3(0.31, 0.39, 0.38) * speck * light * inside * 0.23;

    if scene.anchor.w>0.5 {
        let art=prop_sample(p,vec4(scene.anchor.xy,28.0,7.0));
        color=mix(color,vec3(pow(max(art.r,0.0),0.65)*(light*0.62+0.055*burn)),art.a);
        let da=distance(p,placed_flame);
        color+=lantern_tint*(exp(-da*da/1.1)*0.80+exp(-da*da/23.0)*0.028)*flicker*burn;
    }
    if scene.tools.y>0.0 {
        let lowering=1.0-clamp(scene.tools.y/1.15,0.0,1.0);
        let foot=mix(scene.player.xy+vec2(scene.motion.y*9.0,20.0),scene.placement.xy,lowering);
        let art=prop_sample(p,vec4(foot,28.0,7.0));
        color=mix(color,vec3(pow(max(art.r,0.0),0.65)*(light*0.6+0.025)),art.a);
    }
    // Forty-eight rigid articulated gait poses, aligned to the same boot contact.
    let v=p-scene.player.xy;
    let body_fade=1.0-smoothstep(1.0,4.0,ending);
    var body_uv=sprite_uv(p,scene.player.xy-vec2(0.0,scene.motion.z*0.45),vec2(45.0,60.0),select(scene.motion.y,1.0,vertical));
    let upper=clamp((0.9-body_uv.y)/0.6,0.0,1.0);
    body_uv+=vec2(-idle_sway,idle_breath)*upper;
    if within(body_uv) {
        var art=animate(traveller_texture,traveller_sampler,body_uv,phase,0.0,6.0);
        if vertical {
            art=animate(vertical_texture,vertical_sampler,body_uv,phase,select(0.0,48.0,scene.appearance.y<0.0),12.0);
        }
        if scene.appearance.z<8.0 && scene.feedback.w>0.15 {
            let landing=ceil(scene.motion.x/38.0)%2.0;
            let first=select(landing*24.0,select(48.0,72.0,scene.appearance.y<0.0),vertical);
            let pose=first+floor(clamp((scene.feedback.w-0.15)/0.85,0.0,1.0)*23.0);
            art=textureSampleLevel(idle_texture,idle_sampler,atlas_uv(body_uv,pose,8.0,12.0),0.0);
        }
        let body_light=0.012+brightness*0.17+pulse_light*0.20+scene.feedback.x*0.35;
        color=mix(color,vec3(pow(max(art.r,0.0),0.58)*body_light),art.a*body_fade);
    }
    let ready=1.0-smoothstep(0.0,3.8,scene.clock.w);
    let core=exp(-dot(p-lantern,p-lantern)/1.5)*(1.0-scene.clock.y*0.38*(0.5+sin(time*16.0)*0.5));
    color+=lantern_tint*core*(0.015+brightness*(0.45+ready*0.15))*body_fade*flicker;
    color+=lantern_tint*exp(-dot(p-lantern,p-lantern)/55.0)*brightness*0.045*flicker;
    // Only props whose feet lie in front of the traveller mask their body.
    for (var i=0u;i<u32(scene.motion.w);i+=1u) {
        let prop=scene.props[i];
        if prop.y<scene.player.y-10.0 && within(body_uv) {
            let art=prop_sample(p,prop);
            color=mix(color,vec3(pow(max(art.r,0.0),0.65)*(light+(personal+pulse_light)*0.18)*0.76+0.001),art.a*0.14);
        }
    }
    let progress_angle = (atan2(v.y, v.x) + 3.14159265) / TAU;
    if scene.controls.y > 0.01 && progress_angle < clamp(scene.controls.y, 0.0, 1.0) {
        color += vec3(0.44, 0.51, 0.39) * line(length(v) - 23.0, 0.55) * 0.5;
    }
    for (var k = 0; k < i32(scene.clock.z); k += 1) {
        let a = time * 0.65 + f32(k) * 3.1;
        let follower = scene.player.xy + vec2(cos(a) * 23.0, sin(a * 1.13) * 19.0);
        let df = distance(p, follower);
        color += vec3(0.50, 0.52, 0.30) * exp(-df * df / 3.0) * 0.24 * body_fade;
    }
    color *= 1.0 - scene.controls.z * 0.55;
    if ending > 0.0 {
        color *= 1.0 - smoothstep(6.0, 11.0, ending) * 0.96;
        for (var k = 0; k < 1 + i32(scene.clock.z); k += 1) {
            let a = f32(k) * TAU / (1.0 + scene.clock.z) + 0.9;
            let emergence = smoothstep(2.0 + f32(k) * 0.18, 4.0 + f32(k) * 0.18, ending);
            let travel = min(max(ending - 2.0, 0.0), 15.0);
            let mote = scene.player.xy + vec2(cos(a), sin(a)) * (23.0 + travel * (15.0 + f32(k) * 1.7)) + vec2(sin(time * 0.4 + f32(k)) * 3.0, cos(time * 0.3 + f32(k)) * 5.0);
            let dm = distance(p, mote);
            color += vec3(0.57, 0.51, 0.32) * emergence * (exp(-dm * dm / 11.0) * 0.68 + exp(-dm * dm / 180.0) * 0.045);
        }
    }

    color = mix(vec3(0.00035, 0.00055, 0.00065), color, smoothstep(-90.0, -45.0, d));
    color+=vec3(shadow_rim*pulse_light*0.04)*(1.0-inside);
    let vignette = 1.0 - smoothstep(0.3, 0.79, length((uv - 0.5) * vec2(1.0, 0.86)));
    color *= 0.49 + 0.51 * vignette;
    let strain=smoothstep(0.55,0.94,scene.clock.y);
    let heartbeat=pow(max(0.0,sin(time*(7.0+strain*4.0))),5.0);
    color*=1.0-(1.0-vignette)*strain*(0.30+heartbeat*0.40);
    color+=vec3(0.30,0.21,0.13)*scene.feedback.x*(0.09+(1.0-vignette)*0.27);
    color+=vec3(0.055,0.042,0.027)*(1.0-vignette)*strain*heartbeat;
    let grain = hash(floor(uv * scene.view.zw * 1.4) + floor(time * 13.0)) - 0.5;
    color += grain * 0.0010;
    return vec4(max(color,vec3(0.0)), 1.0);
}
