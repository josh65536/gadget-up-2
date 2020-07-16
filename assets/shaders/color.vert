uniform mat4 worldViewProjectionMatrix;

in vec3 position;
in vec2 offset;
in vec3 color;

out vec3 col;

void main()
{
  col = color;
  gl_Position = worldViewProjectionMatrix * vec4(position.xy + offset, position.z, 1.0);
}
