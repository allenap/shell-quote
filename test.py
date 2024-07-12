from itertools import combinations

features = "bstr", "bash", "fish", "sh"

def power_set(input):
    for length in range(0, len(input) + 1):
        yield from combinations(input, length)
    
if __name__ == '__main__':
    for combo in power_set(features):
        combo_s = "''" if len(combo) == 0 else ",".join(combo)
        print(f"cargo build --no-default-features --features {combo_s}")
        print(f"cargo test  --no-default-features --features {combo_s} --quiet --no-fail-fast")
