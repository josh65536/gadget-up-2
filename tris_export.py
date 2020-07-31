import bpy

filename = "/home/joshua/rust/gadget-up-2/assets/models/save.tris"

obj = bpy.context.active_object
dg = bpy.context.evaluated_depsgraph_get()

obj = obj.evaluated_get(dg)
mesh = obj.to_mesh()

loop_map = {l.vertex_index: i for i, l in enumerate(mesh.loops)}

result = "Triangles::new(vec!["

for (i, v) in enumerate(mesh.vertices):
    v.co.z = 0.0
    
    result += ("Vertex::new(vec3"
        + str(tuple(v.co))
        + ", vec3(0., 0., 0.), vec4"
        + str(tuple(mesh.vertex_colors[0].data[loop_map[i]].color))
        + ", []),")
        
result += "], vec!" + str([v for p in mesh.polygons for v in p.vertices]) + ",)"

with open(filename, "w") as f:
    f.write(result)