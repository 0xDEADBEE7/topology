/** Pins where units start, what they are named and which lines they own. */
export const schema = {
  /** Declared with an object key. */
  viaPair: Joi.string()
    .allow(null)
    .default(() => "x"),
};

/** Declared by assignment. */
exports.viaAssignment = list
  .map((item) => item.id);

class UserTable {
  columns = { render: (user) => user.name };
  /** Declared as a class field. */
  visible = items
    .filter((item) => item.shown);
  render() {
    return 1;
  }
}

@Component({ factory: () => 1 })
class Widget extends mixin({ describe: () => "widget" }) {}

export const Button = memo(
  // a comment is not an argument
  (props) => props.label,
);

export const withAuth = (Component) => function WithAuth(props) {
  if (!props.user) {
    return null;
  }
  
　// spaces before a comment
  return Component(props);
};

const loaded = fetch(url).then((response) => {
  function parse(body) {
    return body;
  }
  return parse(response);
}).catch(() => null);
