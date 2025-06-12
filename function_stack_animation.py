from manim import *


class FunctionCallStackAnimation(Scene):
  def construct(self):
    # 0. Set up layout
    self.camera.background_color = "#1E1E1E"

    # Right side: Code
    code_str = """def main():
    a = 1
    b = 2
    c = add(a,b)
    log(c)

def add(a,b):
    return a+b

def log(value):
    print(value)"""
    code = Code(
      code_string=code_str,
      tab_width=4,
      language="Python",
      formatter_style="monokai",
      paragraph_config={
        "line_spacing": 0.6,
        "font": "Monospace",
      },
    ).to_edge(RIGHT, buff=1)
    self.add(code)

    # Left side: Stack
    stack_height = 6
    stack_width = 3
    stack = Rectangle(
      height=stack_height,
      width=stack_width,
      color=WHITE,
      stroke_width=2,
    ).to_edge(LEFT, buff=1)
    stack_top_label = Text("High Address", font_size=20).next_to(stack, UP, buff=0.2)
    stack_bottom_label = Text("Low Address", font_size=20).next_to(
      stack, DOWN, buff=0.2
    )
    self.add(stack, stack_top_label, stack_bottom_label)

    # SP Pointer
    sp_arrow = Arrow(
      start=stack.get_corner(UL) + RIGHT * (stack_width + 0.5),
      end=stack.get_corner(UL) + RIGHT * 0.1,
      buff=0,
      color=YELLOW,
    )
    sp_label = Text("SP", font_size=24, color=YELLOW).next_to(sp_arrow, LEFT, buff=0.2)
    sp_group = VGroup(sp_arrow, sp_label)

    # Initial Animation
    self.play(
      Write(code), Create(stack), Write(stack_top_label), Write(stack_bottom_label)
    )
    self.play(FadeIn(sp_group))
    self.wait(1)

    # Helper for highlighter
    def create_highlighter(line_number):
      return SurroundingRectangle(
        code[2][line_number], color=BLUE, buff=0.1, fill_color=BLUE, fill_opacity=0.2
      )

    highlighter = create_highlighter(0)

    # Animation steps based on the script

    # Main function call
    self.play(Create(highlighter))
    self.wait(2)

    main_frame_height = 2
    main_frame = Rectangle(
      height=main_frame_height,
      width=stack_width,
      color=BLUE,
      fill_color=BLUE,
      fill_opacity=0.5,
    ).align_to(stack, UL)
    main_label = Text("main()", font_size=24).next_to(
      main_frame,
      RIGHT,
    )

    self.play(sp_group.animate.shift(DOWN * main_frame_height))
    self.play(FadeIn(main_frame), Write(main_label))
    self.wait(1)

    lr_main = Text("lr(return address)", font_size=18).move_to(
      main_frame.get_top() + DOWN * 0.3
    )
    self.play(Write(lr_main), Flash(lr_main, color=YELLOW))
    self.wait(2)

    # Main function internal variable allocation
    self.play(highlighter.animate.become(create_highlighter(1)))
    var_a = Text("a = 1", font_size=20).move_to(lr_main.get_center() + DOWN * 0.5)
    self.play(Write(var_a), Flash(var_a, color=YELLOW))
    self.wait(2)

    self.play(highlighter.animate.become(create_highlighter(2)))
    var_b = Text("b = 2", font_size=20).move_to(var_a.get_center() + DOWN * 0.5)
    self.play(Write(var_b), Flash(var_b, color=YELLOW))
    self.wait(2)

    # Add function call
    self.play(highlighter.animate.become(create_highlighter(3)))
    self.wait(1)
    self.play(highlighter.animate.become(create_highlighter(6)))
    self.wait(2)

    add_frame_height = 1.8
    add_frame = Rectangle(
      height=add_frame_height,
      width=stack_width,
      color=GREEN,
      fill_color=GREEN,
      fill_opacity=0.5,
    ).next_to(main_frame, DOWN, buff=0)
    add_label = Text("add()", font_size=24).next_to(
      add_frame,
      RIGHT,
    )

    self.play(sp_group.animate.shift(DOWN * add_frame_height))
    self.play(FadeIn(add_frame), Write(add_label))

    lr_add = Text("lr(return address)", font_size=18).move_to(
      add_frame.get_top() + DOWN * 0.3
    )
    param_a = Text("a = 1 (copy)", font_size=18).move_to(
      lr_add.get_center() + DOWN * 0.5
    )
    param_b = Text("b = 2 (copy)", font_size=18).move_to(
      param_a.get_center() + DOWN * 0.5
    )
    self.play(Write(lr_add), Write(param_a), Write(param_b))
    self.wait(2)

    # Add function execution and return
    self.play(highlighter.animate.become(create_highlighter(7)))
    self.wait(2)

    self.play(
      FadeOut(add_frame),
      FadeOut(add_label),
      FadeOut(lr_add),
      FadeOut(param_a),
      FadeOut(param_b),
    )
    self.play(sp_group.animate.shift(UP * add_frame_height))
    self.play(highlighter.animate.become(create_highlighter(3)))
    self.wait(2)

    # Main function continues
    var_c = Text("c = 3", font_size=20).move_to(var_b.get_center() + DOWN * 0.5)
    self.play(Write(var_c), Flash(var_c, color=YELLOW))
    self.wait(2)

    # Log function call
    self.play(highlighter.animate.become(create_highlighter(4)))
    self.wait(1)
    self.play(highlighter.animate.become(create_highlighter(9)))
    self.wait(2)

    log_frame_height = 1.5
    log_frame = Rectangle(
      height=log_frame_height,
      width=stack_width,
      color=PURPLE,
      fill_color=PURPLE,
      fill_opacity=0.5,
    ).next_to(main_frame, DOWN, buff=0)
    log_label = Text("log()", font_size=24).next_to(
      log_frame,
      RIGHT,
    )

    self.play(sp_group.animate.shift(DOWN * log_frame_height))
    self.play(FadeIn(log_frame), Write(log_label))

    lr_log = Text("lr(return address)", font_size=18).move_to(
      log_frame.get_top() + DOWN * 0.3
    )
    param_val = Text("value = 3", font_size=18).move_to(
      lr_log.get_center() + DOWN * 0.5
    )
    self.play(Write(lr_log), Write(param_val))
    self.wait(2)

    self.play(highlighter.animate.become(create_highlighter(10)))
    self.wait(2)

    self.play(
      FadeOut(log_frame), FadeOut(log_label), FadeOut(lr_log), FadeOut(param_val)
    )
    self.play(sp_group.animate.shift(UP * log_frame_height))
    self.wait(1)

    # Program end
    self.play(
      FadeOut(main_frame),
      FadeOut(main_label),
      FadeOut(lr_main),
      FadeOut(var_a),
      FadeOut(var_b),
      FadeOut(var_c),
    )
    self.play(sp_group.animate.shift(UP * main_frame_height))
    self.play(FadeOut(highlighter))
    self.wait(3)
