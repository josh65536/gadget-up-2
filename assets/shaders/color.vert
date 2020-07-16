uniform mat4 worldViewProjectionMatrix;

in vec3 position;
in vec3 offset;
in vec3 color;

out vec3 col;

void main()
{
  col = color;
  gl_Position = worldViewProjectionMatrix * vec4(position + offset, 1.0);
}
