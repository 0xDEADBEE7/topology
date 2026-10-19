"""Exercises Python naming, documentation and cognitive complexity rules."""


def sum_of_primes(maximum):
    """Adds the primes up to maximum (SonarSource's 7, less the labelled jump)."""
    total = 0
    for i in range(1, maximum + 1):
        for j in range(2, i):
            if i % j == 0:
                break
        total += i
    return total


def get_words(number):
    match number:
        case 1:
            return "one"
        case 2:
            return "a couple"
        case _:
            return "lots"


def mixed(a, b, c):
    if a and b or c:
        return 1
    elif a:
        return 2
    else:
        f = lambda y: y if y > 1 else 0
        return f(3)


class Shape:
    """A shape."""

    @property
    def area(self):
        "bare string"
        try:
            return 1
        except ValueError:
            return 2


def page(title):
    return """\
<html>
  <title>%s</title>
</html>
""" % title
