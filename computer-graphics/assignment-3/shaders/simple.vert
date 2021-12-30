#version 430 core

layout (location = 0 ) in vec3 position;
layout (location = 2 ) in vec4 color;
layout ( location = 3) uniform mat4 transformation;
layout (location = 6) in vec3 normals;
layout (location = 7) uniform mat4 model;

out VS_OUTPUT{
   vec4 color;
   vec3 normals;
   mat4 model;
} OUT;




void main()
{
   gl_Position = transformation*vec4(position, 1.0f);
   OUT.color = color;
   OUT.normals = normals;
   OUT.model = model;
  
   
}