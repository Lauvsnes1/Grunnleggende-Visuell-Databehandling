#version 430 core

layout (location = 0 ) in vec3 position;
layout (location = 2 ) in vec4 color;
layout ( location = 3) uniform mat4 transformation;

out VS_OUTPUT{
   vec4 color;
} OUT;


void main()
{
   gl_Position = transformation*vec4(position, 1.0f);
   OUT.color = color;
  
   
}