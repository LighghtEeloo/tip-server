import re
import os
import time
import typst
from jsonrpcserver import method, serve, Success, Error

# Path to the .typ file
file_path = 'hello.typ'
compiler = typst.Compiler(file_path)
default_preamble = ""

# TODO: allow customization here
default_extra = '#set page(width: auto, height: auto)\n#set page(margin: (x: 0cm, y: 0.2cm))\n'
blocked_extra = '#set page(width: 16cm, height: auto)\n#set page(margin: (x: 0cm, y: 0.2cm))\n'

@method
def set_default_preamble(preamble):
    default_preamble = preamble
    print(f"default preamble set to {default_preamble}")
    return Success(f"default preamble set to {default_preamble}")


@method
def compile_typst_document_batch(color, preamble, batch):
    """
    color: foreground color of math
    preamble: preamble
    batch: should be a list of [content, hashval]
    """
    print(f"received batch request of length {len(batch)}")
    print(batch)
    count = 0
    start = time.time()
    print(f"Setting color {color}")
    extra = f'#show math.equation: set text(rgb("{color}"))\n'

    for content, hashval in batch:
        # Update the content of the .typ file
        if re.match(r'\$\s', content):
            print(f"got blocked content {content}")
            extra += blocked_extra
        else:
            print(f"got content {content}")
            extra += default_extra
        with open(file_path, 'w') as f:
            f.write(extra)
            f.write(preamble)
            f.write(content)
        svg_path = os.path.abspath(os.path.join('output', f'{hashval}.svg'))

        # Compile the .typ file to SVG format
        svg = compiler.compile(format='svg')

        # Return the result
        if svg:
            with open(svg_path, 'wb') as f:
                f.write(svg)
            count += 1
            print(f"finised processing {count}-th fragment")
        else:
            return Error(f"failed to create {count}-th svg.")
    end = time.time()
    return Success(f"All {count} fragments compiled after {end-start} seconds.")


@method
def compile_fragment_live(color, preamble, content, hashval):
    """
    color: foreground color of math
    preamble: preamble
    batch: should be a list of [content, hashval]
    """
    print(f"trying to live compile {content}")
    start = time.time()
    print(f"Setting color {color}")
    extra = f'#show math.equation: set text(rgb("{color}"))\n'

    # Update the content of the .typ file
    if re.match(r'\$\s', content):
        print(f"got blocked content {content}")
        extra += blocked_extra
    else:
        print(f"got content {content}")
        extra += default_extra

    with open(file_path, 'w') as f:
        f.write(extra)
        f.write(preamble)
        f.write(content)
        svg_path = os.path.abspath(os.path.join('output', f'live-{hashval}.svg'))

    # Compile the .typ file to SVG format
    svg = compiler.compile(format='svg')

    # Return the result
    if svg:
        with open(svg_path, 'wb') as f:
            f.write(svg)
            print(f"wrote svg to {svg_path}")
    else:
        return Error(f"failed to create {count}-th svg.")
    end = time.time()
    print(f"fragment live-compiled after {end-start} seconds.")
    return Success(f"fragments compiled after {end-start} seconds.")


@method
def delete_svg_files(batch):
    """
    batch should be a list of hashval
    """
    print(f"received batch request of length {len(batch)}")
    print(batch)
    count = 0
    for hashval in batch:
        svg_path = os.path.join('output', f'{hashval}.svg')
        if os.path.exists(svg_path):
            os.remove(svg_path)
            count += 1
    return Success(f"Removed {count}/{len(set(batch))} svg files for {len(batch)} fragments.")


@method
def trivial(name):
    return Success(f"Hi {name}")


# Run the JSON-RPC server
if __name__ == "__main__":
    serve()
