export interface SqlSample {
  name: string;
  sql: string;
}

export const samples: SqlSample[] = [
  {
    name: "Simple SELECT",
    sql: "select id, name, email from users where active = true order by name asc limit 10;",
  },
  {
    name: "JOIN",
    sql: "select u.id, u.name, o.total from users u inner join orders o on u.id = o.user_id where o.status = 'completed' order by o.total desc;",
  },
  {
    name: "Subquery",
    sql: "select name, email from users where id in (select user_id from orders where total > 100 and created_at > '2024-01-01');",
  },
  {
    name: "CTE (WITH)",
    sql: "with active_users as (select id, name from users where active = true), user_orders as (select user_id, count(*) as order_count, sum(total) as total_spent from orders group by user_id) select au.name, uo.order_count, uo.total_spent from active_users au join user_orders uo on au.id = uo.user_id order by uo.total_spent desc;",
  },
  {
    name: "UNION",
    sql: "select id, name, 'customer' as type from customers where active = true union all select id, name, 'supplier' as type from suppliers where active = true order by name;",
  },
  {
    name: "INSERT",
    sql: "insert into users (name, email, role, created_at) values ('Alice', 'alice@example.com', 'admin', now()), ('Bob', 'bob@example.com', 'user', now());",
  },
  {
    name: "UPDATE with JOIN",
    sql: "update orders set status = 'archived', updated_at = now() where created_at < '2023-01-01' and status = 'completed';",
  },
  {
    name: "CREATE TABLE",
    sql: "create table if not exists products (id serial primary key, name varchar(255) not null, description text, price decimal(10, 2) not null default 0.00, category_id integer references categories(id), created_at timestamp default current_timestamp, updated_at timestamp default current_timestamp);",
  },
  {
    name: "Window Function",
    sql: "select name, department, salary, rank() over (partition by department order by salary desc) as dept_rank, avg(salary) over (partition by department) as dept_avg from employees where active = true;",
  },
  {
    name: "Nested Subquery",
    sql: "select d.name as department, (select count(*) from employees e where e.department_id = d.id) as emp_count, (select avg(salary) from employees e where e.department_id = d.id) as avg_salary from departments d where exists (select 1 from employees e where e.department_id = d.id and e.active = true) order by emp_count desc;",
  },
];
