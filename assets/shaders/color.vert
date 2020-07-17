uniform mat4 worldViewProjectionMatrix;

in vec3 position;
in vec3 offset;
in vec4 color;

out vec4 col;

void main()
{
  col = color;
  gl_Position = worldViewProjectionMatrix * vec4(position + offset, 1.0);
}
