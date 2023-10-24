@group(0)
@binding(0)
var<storage, read> input: array<u32>;
@group(0)
@binding(1)
var<storage, write> output: array<u32>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	for(var i: u32 = 0u; i < 5u; i++) {
		if (i <= arrayLength(&input)) {
			output[i] = 100u;
		} else {
			output[i] = i;
		}
	}
}
