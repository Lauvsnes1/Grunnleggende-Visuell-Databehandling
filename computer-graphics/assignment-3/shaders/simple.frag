#version 430 core



in VS_OUTPUT{
    vec4 color;
    vec3 normals;
    mat4 model;
} IN;


out vec4 color;

void main()
{   
    vec3 dynamic_normals = normalize(mat3x3(IN.model) * IN.normals);
    vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    float diff = max(0,dot(dynamic_normals,-lightDirection));
    color = vec4(IN.color.rgb*diff, IN.color.a);
}